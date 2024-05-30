// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use bumpalo::collections::Vec as BumpVec;
use std::cell::RefCell;
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
pub struct Queue<'d> {
    handle: Handle<VkQueue>,
    device: &'d Device<'d>,
    resources: CleanupQueue,
    scratch: Exclusive<bumpalo::Bump>,
}

impl Device<'_> {
    pub(crate) fn queue(
        self: &Self, family_index: u32, queue_index: u32,
    ) -> Queue {
        let mut handle = None;
        unsafe {
            (self.fun.get_device_queue)(
                self.handle(),
                family_index,
                queue_index,
                &mut handle,
            );
        }
        Queue {
            handle: handle.unwrap(),
            device: self,
            resources: CleanupQueue::new(100),
            scratch: Exclusive::new(bumpalo::Bump::new()),
        }
    }
}

impl Queue<'_> {
    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkQueue> {
        self.handle.borrow()
    }
    /// Mutably borrows the inner Vulkan handle.
    pub fn mut_handle(&mut self) -> Mut<VkQueue> {
        self.handle.borrow_mut()
    }
}

pub struct SubmitScope<'env> {
    handle: RefCell<Mut<'env, VkQueue>>,
    device: &'env Device<'env>,
    scratch: &'env bumpalo::Bump,
}

impl Drop for SubmitScope<'_> {
    fn drop(&mut self) {
        if unsafe {
            (self.device.fun.queue_wait_idle)(
                self.handle.borrow_mut().reborrow_mut(),
            )
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

impl<'d> Queue<'d> {
    pub fn scope<'env, F>(&'env mut self, f: F)
    where
        F: for<'scope> FnOnce(&'scope SubmitScope<'env>),
    {
        f(&SubmitScope {
            handle: self.handle.borrow_mut().into(),
            device: self.device,
            scratch: self.scratch.get_mut(),
        });
    }

    /// Scoped submit that allows new commands to be submitted before the
    /// previous ones complete. This can improve device utilization because
    /// draining and refilling work queues on the device takes some time.
    pub fn double_buffered_scope<'env, F, T>(
        &'env mut self, (_a, _b): (&'env mut T, &'env mut T), _f: F,
    ) where
        F: for<'scope> FnMut(&'scope SubmitScope<'env>, &'scope mut T) -> bool,
    {
    }

    #[doc = crate::man_link!(vkQueueWaitIdle)]
    pub fn wait_idle(&mut self) -> Result<()> {
        unsafe { (self.device.fun.queue_wait_idle)(self.handle.borrow_mut())? };
        self.resources.new_cleanup().cleanup();
        Ok(())
    }
}

#[doc = crate::man_link!(VkSubmitInfo)]
pub enum Submit<'a> {
    Wait(&'a Semaphore<'a>, PipelineStageFlags),
    Command(&'a mut CommandBuffer<'a>),
    Signal(&'a Semaphore<'a>),
}

impl<'env> SubmitScope<'env> {
    pub fn submit<'scope>(
        &'scope self, infos: impl IntoIterator<Item = Submit<'env>>,
    ) {
        let mut builder = SubmitBuilder {
            scratch: self.scratch,
            submits: bumpalo::vec![in self.scratch],
            wait: bumpalo::vec![in self.scratch],
            masks: bumpalo::vec![in self.scratch],
            commands: bumpalo::vec![in self.scratch],
            signal: bumpalo::vec![in self.scratch],
        };
        for info in infos {
            match info {
                Submit::Wait(semaphore, wait_stage_mask) => {
                    builder.add_wait(semaphore.handle(), wait_stage_mask)
                }
                Submit::Command(cmd) => builder.add_command(cmd.handle_mut()),
                Submit::Signal(semaphore) => {
                    builder.add_signal(semaphore.handle())
                }
            }
        }
        builder.advance();

        unsafe {
            (self.device.fun.queue_submit)(
                self.handle.borrow_mut().reborrow_mut(),
                builder.submits.len() as u32,
                Array::from_slice(&builder.submits),
                None,
            )
        }
        .unwrap();
    }
}

struct SubmitBuilder<'bump> {
    scratch: &'bump bumpalo::Bump,
    submits: BumpVec<'bump, VkSubmitInfo<'bump, Null>>,
    wait: BumpVec<'bump, Ref<'bump, VkSemaphore>>,
    masks: BumpVec<'bump, PipelineStageFlags>,
    commands: BumpVec<'bump, Mut<'bump, VkCommandBuffer>>,
    signal: BumpVec<'bump, Ref<'bump, VkSemaphore>>,
}

impl<'bump> SubmitBuilder<'bump> {
    fn advance(&mut self) {
        fn take<'b, T>(
            vec: &mut bumpalo::collections::Vec<'b, T>, bump: &'b bumpalo::Bump,
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
    fn add_wait(
        &mut self, wait: Ref<'bump, VkSemaphore>, mask: PipelineStageFlags,
    ) {
        if !self.commands.is_empty() || !self.signal.is_empty() {
            self.advance()
        }
        self.wait.push(wait);
        self.masks.push(mask);
    }
    fn add_command(&mut self, command: Mut<'bump, VkCommandBuffer>) {
        if !self.signal.is_empty() {
            self.advance()
        }
        self.commands.push(command);
    }
    fn add_signal(&mut self, signal: Ref<'bump, VkSemaphore>) {
        self.signal.push(signal);
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
