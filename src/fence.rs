// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::cleanup_queue::Cleanup;
use crate::device::Device;
use crate::error::Result;
use crate::types::*;

/// A
#[doc = concat!(crate::spec_link!("fence", "7", "synchronization-fences"), ".")]
/// When submitted to a [`Queue`](crate::vk::Queue), becomes a [`PendingFence`].
#[derive(Debug)]
pub struct Fence<'d> {
    handle: Option<Handle<VkFence>>,
    device: &'d Device<'d>,
}

/// A
#[doc = crate::spec_link!("fence", "7", "synchronization-fences")]
/// with a signal operation pending.
#[derive(Debug)]
#[must_use = "Dropping a pending fence leaks it."]
pub struct PendingFence<'d> {
    handle: Handle<VkFence>,
    device: &'d Device<'d>,
    resources: Cleanup,
}

impl<'d> Fence<'d> {
    #[doc = crate::man_link!(vkCreateFence)]
    pub fn new(device: &'d Device) -> Result<Self> {
        let mut handle = None;
        unsafe {
            (device.fun.create_fence)(
                device.handle(),
                &Default::default(),
                None,
                &mut handle,
            )?;
        }
        Ok(Self { handle, device: device.clone() })
    }
}

impl Drop for Fence<'_> {
    fn drop(&mut self) {
        if let Some(handle) = &mut self.handle {
            unsafe {
                (self.device.fun.destroy_fence)(
                    self.device.handle(),
                    handle.borrow_mut(),
                    None,
                )
            }
        }
    }
}

impl<'d> Fence<'d> {
    /// Borrows the inner Vulkan handle.
    pub fn mut_handle(&mut self) -> Mut<VkFence> {
        self.handle.as_mut().unwrap().borrow_mut()
    }
    pub(crate) fn into_pending(
        mut self, resources: Cleanup,
    ) -> PendingFence<'d> {
        PendingFence {
            handle: self.handle.take().unwrap(),
            device: self.device,
            resources,
        }
    }
}

impl<'d> PendingFence<'d> {
    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkFence> {
        self.handle.borrow()
    }
    /// Waits for the fence, decrements the reference count of any objects
    /// (including [`CommandPools`](crate::vk::CommandPool)) submitted to
    /// the queue, and resets the fence.
    #[doc = crate::man_link!(vkWaitForFences)]
    pub fn wait(mut self) -> Result<Fence<'d>> {
        unsafe {
            (self.device.fun.wait_for_fences)(
                self.device.handle(),
                1,
                (&[self.handle.borrow()]).into(),
                true.into(),
                u64::MAX,
            )?;
        }
        self.resources.cleanup();
        unsafe {
            (self.device.fun.reset_fences)(
                self.device.handle(),
                1,
                // Safe because the the outer structure is owned here
                (&[self.handle.borrow_mut()]).into(),
            )?;
        }
        Ok(Fence { handle: Some(self.handle), device: self.device })
    }
}
