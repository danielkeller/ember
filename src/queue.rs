// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use bumpalo::collections::Vec as BumpVec;
use std::fmt::Debug;

use crate::cleanup_queue::CleanupQueue;
use crate::command_buffer::CommandBuffer;
use crate::device::Device;
use crate::error::Result;
use crate::exclusive::Exclusive;
use crate::ffi::{Array, Null};
use crate::semaphore::Semaphore;
use crate::types::*;
use crate::vk::PipelineStageFlags;

/// A queue.
///
/// Returned from [`Device::new`].
///
/// Each queue interally holds references to the resources submitted to it. This
/// includes [`CommandBuffer`](crate::vk::CommandBuffer)s,
/// [`CommandPool`](crate::vk::CommandPool)s, [`Semaphore`]s, and
/// [`SwapchainKHR`](crate::vk::ext::SwapchainKHR)s. The resources cannot be
/// freed (or [`reset`](crate::vk::CommandPool::reset()) in the case of command
/// pools) until the queue is done with them. This happens when either
/// * [`Queue::wait_idle`] is called.
/// * [`PendingFence::wait`](PendingFence::wait()) is called on a fence passed
/// to [`Queue::submit_with_fence`](Queue::submit_with_fence()).
/// * A semaphore is passed to `submit` in [`SubmitInfo::signal`], then passed
/// to another queue in [`SubmitInfo::wait`], (and so on) and on the last queue
/// one of the first two things is done.
#[derive(Debug)]
pub struct Queue {
    handle: Handle<VkQueue>,
    device: Device,
    resources: CleanupQueue,
    scratch: Exclusive<bumpalo::Bump>,
}

impl Device {
    pub(crate) fn queue(
        self: &Self, family_index: u32, queue_index: u32,
    ) -> Queue {
        let mut handle = None;
        unsafe {
            (self.fun().get_device_queue)(
                self.handle(),
                family_index,
                queue_index,
                &mut handle,
            );
        }
        Queue {
            handle: handle.unwrap(),
            device: self.clone(),
            resources: CleanupQueue::new(100),
            scratch: Exclusive::new(bumpalo::Bump::new()),
        }
    }
}

impl Queue {
    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkQueue> {
        self.handle.borrow()
    }
    /// Mutably borrows the inner Vulkan handle.
    pub fn mut_handle(&mut self) -> Mut<VkQueue> {
        self.handle.borrow_mut()
    }
}

/// A builder for queue submissions. Nothing is actually submitted until either
/// the scope ends, or [SubmitScope::submit] is called. Any values that are used
/// by the device must live for `'scope`.
pub struct SubmitScope<'scope> {
    handle: Mut<'scope, VkQueue>,
    device: Device,
    scratch: &'scope bumpalo::Bump,
    submits: BumpVec<'scope, VkSubmitInfo<'scope, Null>>,
    wait: BumpVec<'scope, Ref<'scope, VkSemaphore>>,
    masks: BumpVec<'scope, PipelineStageFlags>,
    commands: BumpVec<'scope, Mut<'scope, VkCommandBuffer>>,
    signal: BumpVec<'scope, Ref<'scope, VkSemaphore>>,
}

impl Drop for SubmitScope<'_> {
    fn drop(&mut self) {
        if unsafe {
            (self.device.fun().queue_wait_idle)(self.handle.reborrow_mut())
        }
        .is_err()
        {
            // Potentially in some cases (eg device loss) this is actually
            // recoverable since the other destructors will fail in the same
            // way.
            eprintln!("vkQueueWaitIdle failed");
            std::process::abort();
        }
    }
}

impl Queue {
    fn submit_scope(&mut self) -> SubmitScope<'_> {
        let scratch = self.scratch.get_mut();
        SubmitScope {
            handle: self.handle.borrow_mut().into(),
            device: self.device.clone(),
            scratch,
            submits: bumpalo::vec![in scratch],
            wait: bumpalo::vec![in scratch],
            masks: bumpalo::vec![in scratch],
            commands: bumpalo::vec![in scratch],
            signal: bumpalo::vec![in scratch],
        }
    }

    pub fn scope<'scope, F, R>(&'scope mut self, f: F) -> R
    where
        F: FnOnce(&mut SubmitScope<'scope>) -> R,
    {
        let mut submit_scope = self.submit_scope();
        let result = f(&mut submit_scope);
        submit_scope.submit();
        result
    }

    /// Scoped submit that allows new commands to be submitted before the
    /// previous ones complete. This can improve device utilization because
    /// it avoids draining and refilling the work queues on the device between
    /// submissions. The values that are used by the submission are passed
    /// explicitly, rather than as part of the capture as in [Queue::scope].
    pub fn scope_loop<F, T, R>(&mut self, values: &mut [T], mut f: F) -> R
    where
        // The "escape condition" on FnMut disallows any captures from being
        // borrowed by submission, since they must live for 'scope which
        // outlives the body.
        F: for<'scope> FnMut(
            &mut SubmitScope<'scope>,
            &'scope mut T,
        ) -> std::ops::ControlFlow<R>,
    {
        loop {
            for value in values.iter_mut() {
                let mut submit_scope = self.submit_scope();
                let result = f(&mut submit_scope, value);
                submit_scope.submit();
                // FIXME: dropping the scope drains the queue, don't do that.
                if let std::ops::ControlFlow::Break(v) = result {
                    return v;
                }
            }
        }
    }

    // TODO: Async version of submit_loop that disables the device while it's
    // not running.

    #[doc = crate::man_link!(vkQueueWaitIdle)]
    pub fn wait_idle(&mut self) -> Result<()> {
        unsafe {
            (self.device.fun().queue_wait_idle)(self.handle.borrow_mut())?
        };
        self.resources.new_cleanup().cleanup();
        Ok(())
    }
}

impl<'scope> SubmitScope<'scope> {
    pub fn mut_handle(&mut self) -> Mut<VkQueue> {
        self.handle.reborrow_mut()
    }

    /// Call
    #[doc = crate::man_link!(vkQueueSubmit)]
    /// immediately with all commands so far (if any).
    pub fn submit(&mut self) {
        self.advance();

        if !self.submits.is_empty() {
            unsafe {
                (self.device.fun().queue_submit)(
                    self.handle.reborrow_mut(),
                    self.submits.len() as u32,
                    Array::from_slice(&self.submits),
                    None,
                )
            }
            .unwrap();

            self.submits.clear();
        }
    }

    pub fn wait(
        &mut self, semaphore: &'scope Semaphore, stages: PipelineStageFlags,
    ) {
        if !self.commands.is_empty() || !self.signal.is_empty() {
            self.advance()
        }
        self.wait.push(semaphore.handle());
        self.masks.push(stages);
    }
    pub fn command(&mut self, command: CommandBuffer<'scope>) {
        if !self.signal.is_empty() {
            self.advance()
        }
        self.commands.push(command.into_handle());
    }
    pub fn signal(&mut self, semaphore: &'scope Semaphore) {
        self.signal.push(semaphore.handle());
    }

    fn advance(&mut self) {
        if self.wait.is_empty()
            && self.commands.is_empty()
            && self.signal.is_empty()
        {
            return;
        }

        fn take<'b, T>(
            vec: &mut BumpVec<'b, T>, bump: &'b bumpalo::Bump,
        ) -> &'b [T] {
            std::mem::replace(vec, bumpalo::vec![in bump]).into_bump_slice()
        }

        self.submits.push(VkSubmitInfo {
            wait_semaphores: take(&mut self.wait, self.scratch).into(),
            wait_stage_masks: Array::from_slice(take(
                &mut self.masks,
                self.scratch,
            )),
            command_buffers: take(&mut self.commands, self.scratch).into(),
            signal_semaphores: take(&mut self.signal, self.scratch).into(),
            ..Default::default()
        });
    }
}

#[cfg(test_disabled)]
mod test {
    use crate::vk;

    #[test]
    fn cmd_state() -> vk::Result<()> {
        let (dev, mut q) = crate::test_device()?;
        let mut pool = vk::CommandPoolLifetime::new(&dev, 0)?;
        assert!(pool.reset(Default::default()).is_ok());
        let mut buf = pool.begin().end()?;

        let fence = q.submit_with_fence(
            &mut [vk::SubmitInfo1 {
                commands: &mut [&mut buf],
                ..Default::default()
            }],
            vk::Fence::new(&dev)?,
        )?;
        assert!(q
            .submit_with_fence(
                &mut [vk::SubmitInfo1 {
                    commands: &mut [&mut buf],
                    ..Default::default()
                }],
                vk::Fence::new(&dev)?,
            )
            .is_err());

        assert!(pool.reset(Default::default()).is_err());
        fence.wait()?;
        assert!(pool.reset(Default::default()).is_ok());

        assert!(q
            .submit_with_fence(
                &mut [vk::SubmitInfo1 {
                    commands: &mut [&mut buf],
                    ..Default::default()
                }],
                vk::Fence::new(&dev)?,
            )
            .is_err());

        Ok(())
    }

    #[test]
    fn signaller() -> vk::Result<()> {
        let (dev, mut q) = crate::test_device()?;
        let mut sem = vk::Semaphore::new(&dev)?;
        assert!(q
            .submit_with_fence(
                &mut [vk::SubmitInfo1 {
                    signal: &mut [&mut sem],
                    ..Default::default()
                }],
                vk::Fence::new(&dev)?,
            )
            .is_ok());
        assert!(q
            .submit_with_fence(
                &mut [vk::SubmitInfo1 {
                    signal: &mut [&mut sem],
                    ..Default::default()
                }],
                vk::Fence::new(&dev)?,
            )
            .is_err());
        assert!(q
            .submit_with_fence(
                &mut [vk::SubmitInfo1 {
                    wait: &mut [(&mut sem, Default::default())],
                    ..Default::default()
                }],
                vk::Fence::new(&dev)?,
            )
            .is_ok());
        assert!(q
            .submit_with_fence(
                &mut [vk::SubmitInfo1 {
                    wait: &mut [(&mut sem, Default::default())],
                    ..Default::default()
                }],
                vk::Fence::new(&dev)?,
            )
            .is_err());
        Ok(())
    }

    #[test]
    fn cross_queue_sync() -> vk::Result<()> {
        let inst = vk::Instance::new(&Default::default())?;
        let phy = inst.enumerate_physical_devices()?.remove(0);
        if phy.queue_family_properties().len() < 2 {
            // Can't do the test. Also can't print a message :(
            return Ok(());
        }
        let (dev, mut qs) = vk::Device::new(
            &inst.enumerate_physical_devices()?[0],
            &vk::DeviceCreateInfo {
                queue_create_infos: vk::slice(&[
                    vk::DeviceQueueCreateInfo {
                        queue_priorities: vk::slice(&[1.0]),
                        queue_family_index: 0,
                        ..Default::default()
                    },
                    vk::DeviceQueueCreateInfo {
                        queue_priorities: vk::slice(&[1.0]),
                        queue_family_index: 1,
                        ..Default::default()
                    },
                ]),
                ..Default::default()
            },
        )?;
        let mut q1 = qs.remove(0).remove(0);
        let mut q2 = qs.remove(0).remove(0);

        let mut pool1 = vk::CommandPoolLifetime::new(&dev, 0)?;
        let mut pool2 = vk::CommandPoolLifetime::new(&dev, 0)?;
        let mut buf1 = pool1.begin().end()?;
        let mut buf2 = pool2.begin().end()?;

        let mut sem = vk::Semaphore::new(&dev)?;

        q1.submit(&mut [
            vk::SubmitInfo1 {
                wait: &mut [],
                commands: &mut [&mut buf1],
                signal: &mut [&mut sem],
            },
            vk::SubmitInfo1 {
                commands: &mut [&mut buf2],
                ..Default::default()
            },
        ])?;

        let fence = q2.submit_with_fence(
            &mut [vk::SubmitInfo1 {
                wait: &mut [(&mut sem, vk::PipelineStageFlags::TOP_OF_PIPE)],
                ..Default::default()
            }],
            vk::Fence::new(&dev)?,
        )?;

        assert!(pool1.reset(Default::default()).is_err());
        assert!(pool2.reset(Default::default()).is_err());

        fence.wait()?;
        assert!(pool1.reset(Default::default()).is_ok());
        assert!(pool2.reset(Default::default()).is_err());

        q1.wait_idle()?;
        assert!(pool2.reset(Default::default()).is_ok());

        Ok(())
    }
}
