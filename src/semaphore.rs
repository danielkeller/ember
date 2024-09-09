// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::device::Device;
use crate::error::Result;
use crate::types::*;

// Preventing double-signals requires knowing when things will run, so we don't
// try to.

/// A
#[doc = crate::spec_link!("semaphore", "7", "synchronization-semaphores")]
#[derive(Debug)]
pub struct Semaphore<'d> {
    handle: Handle<VkSemaphore>,
    device: &'d Device<'d>,
}

#[derive(Debug)]
pub struct SignalledSemaphore<'a> {
    handle: Ref<'a, VkSemaphore>,
}

impl<'d> Semaphore<'d> {
    #[doc = crate::man_link!(vkCreateSemaphore)]
    pub fn new(device: &'d Device) -> Result<Self> {
        let mut handle = None;
        unsafe {
            (device.fun.create_semaphore)(
                device.handle(),
                &Default::default(),
                None,
                &mut handle,
            )?;
        }
        Ok(Self { handle: handle.unwrap(), device })
    }

    pub(crate) fn to_signalled(&self) -> SignalledSemaphore {
        SignalledSemaphore { handle: self.handle() }
    }
}

#[cfg(any())]
impl Drop for Semaphore<'_> {
    /// **Warning:** If a semaphore is passed to
    /// [`SwapchainKHR::acquire_next_image`](crate::vk::ext::SwapchainKHR::acquire_next_image())
    /// and then dropped without being waited on, the swapchain and semaphore
    /// will be leaked, since there is no way to know when it can be safely
    /// dropped other than waiting on it.
    fn drop(&mut self) {
        if let Some(SemaphoreSignaller::Swapchain(sc)) = self.signaller.take() {
            // Semaphore incorrectly dropped
            std::mem::forget(sc); // Leak the swapchain
            std::mem::forget(self.inner.clone()); // Leak the semaphore
            eprintln!(
                "Semaphore used with WSI and then freed without being waited on"
            );
        }
        // Dropping an unwaited semaphore is normally fine since for a
        // queue, the signal op is ordered before the fence signal, and the
        // queue and fence take care of the lifetime of the SemaphoreRAII.
    }
}

impl Drop for Semaphore<'_> {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun.destroy_semaphore)(
                self.device.handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

impl Semaphore<'_> {
    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkSemaphore> {
        self.handle.borrow()
    }
    /// Borrows the inner Vulkan handle.
    pub fn mut_handle(&mut self) -> Mut<VkSemaphore> {
        self.handle.borrow_mut()
    }
}

impl<'a> SignalledSemaphore<'a> {
    pub fn into_handle(self) -> Ref<'a, VkSemaphore> {
        self.handle
    }
}
