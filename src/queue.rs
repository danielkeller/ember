// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt::Debug;

use crate::cleanup_queue::CleanupQueue;
use crate::command_buffer::CommandBuffer;
use crate::device::Device;
use crate::error::{Error, Result};
use crate::exclusive::Exclusive;
use crate::fence::{Fence, PendingFence};
use crate::ffi::Array;
use crate::semaphore::{Semaphore, SemaphoreSignaller};
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
                self.borrow(),
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

impl Queue<'_> {
    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkQueue> {
        self.handle.borrow()
    }
    /// Mutably borrows the inner Vulkan handle.
    pub fn mut_handle(&mut self) -> Mut<VkQueue> {
        self.handle.borrow_mut()
    }
    /// Add an item to the queue's cleanup. The value will be dropped when a
    /// fence is submitted and waited on.
    pub(crate) fn add_resource(&mut self, value: Arc<dyn Send + Sync>) {
        self.resources.push(value)
    }
}

impl Drop for Queue<'_> {
    /// Waits for the queue to be idle before dropping resources
    fn drop(&mut self) {
        if let Err(err) = self.wait_idle() {
            self.resources.leak();
            panic!("vkQueueWaitIdle failed: {}", err);
        }
    }
}

#[doc = crate::man_link!(VkSubmitInfo)]
#[derive(Default)]
pub struct SubmitInfo<'a> {
    pub wait: &'a mut [(&'a mut Semaphore<'a>, PipelineStageFlags)],
    pub commands: &'a mut [&'a mut CommandBuffer<'a>],
    pub signal: &'a mut [&'a mut Semaphore<'a>],
}

impl Queue<'_> {
    /// Returns [`Error::InvalidArgument`] if any semaphore in `signal` already
    /// has a signal operation pending, or if any semaphore in `wait` does not,
    /// or if any command buffer is not in the executable state.
    #[doc = crate::man_link!(vkQueueSubmit)]
    pub fn submit_with_fence(
        &mut self, infos: &mut [SubmitInfo<'_>], mut fence: Fence,
    ) -> Result<PendingFence> {
        self.submit_impl(infos, Some(fence.mut_handle()))?;
        Ok(fence.into_pending(self.resources.new_cleanup()))
    }

    /// Returns [`Error::InvalidArgument`] if any semaphore in `signal` already
    /// has a signal operation pending, or if any semaphore in `wait` does not,
    /// or if any command buffer is not in the executable state.
    #[doc = crate::man_link!(vkQueueSubmit)]
    pub fn submit(&mut self, infos: &mut [SubmitInfo<'_>]) -> Result<()> {
        self.submit_impl(infos, None)
    }

    fn submit_impl(
        &mut self, infos: &mut [SubmitInfo<'_>], fence: Option<Mut<VkFence>>,
    ) -> Result<()> {
        for info in infos.iter() {
            for (sem, _) in info.wait.iter() {
                if sem.signaller.is_none() {
                    return Err(Error::InvalidArgument);
                }
            }
            for sem in info.signal.iter() {
                if sem.signaller.is_some() {
                    return Err(Error::InvalidArgument);
                }
            }
        }

        let scratch = self.scratch.get_mut();
        scratch.reset();

        // This needs to stay in a Vec because its destructor is important
        let mut recordings = bumpalo::vec![in scratch];
        let mut vk_infos = bumpalo::vec![in scratch];
        for info in infos.iter_mut() {
            let mut commands = bumpalo::vec![in scratch];
            let mut info_recordings = bumpalo::vec![in scratch];
            for c in info.commands.iter_mut() {
                info_recordings
                    .push(c.lock_resources().ok_or(Error::InvalidArgument)?);
                commands.push(c.borrow_mut()?);
            }
            recordings.push(info_recordings);
            let wait_semaphores = scratch.alloc_slice_fill_iter(
                info.wait.iter().map(|(sem, _)| sem.handle()),
            );
            let wait_stage_masks = scratch.alloc_slice_fill_iter(
                info.wait.iter().map(|(_, mask)| *mask), //
            );
            let signal_semaphores = scratch.alloc_slice_fill_iter(
                info.signal.iter().map(|sem| sem.handle()),
            );
            vk_infos.push(VkSubmitInfo {
                wait_semaphores: wait_semaphores.into(),
                wait_stage_masks: Array::from_slice(wait_stage_masks),
                command_buffers: commands.into_bump_slice().into(),
                signal_semaphores: signal_semaphores.into(),
                ..Default::default()
            });
        }

        unsafe {
            (self.device.fun.queue_submit)(
                self.handle.borrow_mut(),
                vk_infos.len() as u32,
                Array::from_slice(&vk_infos),
                fence,
            )?;
        }
        drop(vk_infos);

        // Everything fallible is done, mark resources as in use
        for (info, recs) in infos.iter_mut().zip(recordings.into_iter()) {
            for (sem, _) in info.wait.iter_mut() {
                self.resources.push(sem.take_signaller());
                self.resources.push(sem.inner.clone());
            }
            self.resources.extend(recs.into_iter());
            for command in info.commands.iter() {
                self.resources.push(command.lock_self());
            }
            for sem in info.signal.iter_mut() {
                sem.signaller = Some(SemaphoreSignaller::Queue(
                    self.resources.new_cleanup(),
                ));
                self.resources.push(sem.inner.clone());
            }
        }
        Ok(())
    }

    #[doc = crate::man_link!(vkQueueWaitIdle)]
    pub fn wait_idle(&mut self) -> Result<()> {
        unsafe { (self.device.fun.queue_wait_idle)(self.handle.borrow_mut())? };
        self.resources.new_cleanup().cleanup();
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::vk;

    #[test]
    fn cmd_state() -> vk::Result<()> {
        let (dev, mut q) = crate::test_device()?;
        let mut pool = vk::CommandPool::new(&dev, 0)?;
        assert!(pool.reset(Default::default()).is_ok());
        let buf = pool.allocate()?;
        let mut buf = pool.begin(buf)?.end()?;

        let fence = q.submit_with_fence(
            &mut [vk::SubmitInfo {
                commands: &mut [&mut buf],
                ..Default::default()
            }],
            vk::Fence::new(&dev)?,
        )?;
        assert!(q
            .submit_with_fence(
                &mut [vk::SubmitInfo {
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
                &mut [vk::SubmitInfo {
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
                &mut [vk::SubmitInfo {
                    signal: &mut [&mut sem],
                    ..Default::default()
                }],
                vk::Fence::new(&dev)?,
            )
            .is_ok());
        assert!(q
            .submit_with_fence(
                &mut [vk::SubmitInfo {
                    signal: &mut [&mut sem],
                    ..Default::default()
                }],
                vk::Fence::new(&dev)?,
            )
            .is_err());
        assert!(q
            .submit_with_fence(
                &mut [vk::SubmitInfo {
                    wait: &mut [(&mut sem, Default::default())],
                    ..Default::default()
                }],
                vk::Fence::new(&dev)?,
            )
            .is_ok());
        assert!(q
            .submit_with_fence(
                &mut [vk::SubmitInfo {
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

        let mut pool1 = vk::CommandPool::new(&dev, 0)?;
        let mut pool2 = vk::CommandPool::new(&dev, 0)?;
        let buf1 = pool1.allocate()?;
        let buf2 = pool2.allocate()?;
        let mut buf1 = pool1.begin(buf1)?.end()?;
        let mut buf2 = pool2.begin(buf2)?.end()?;

        let mut sem = vk::Semaphore::new(&dev)?;

        q1.submit(&mut [
            vk::SubmitInfo {
                wait: &mut [],
                commands: &mut [&mut buf1],
                signal: &mut [&mut sem],
            },
            vk::SubmitInfo { commands: &mut [&mut buf2], ..Default::default() },
        ])?;

        let fence = q2.submit_with_fence(
            &mut [vk::SubmitInfo {
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
