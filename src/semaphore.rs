// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::cleanup_queue::Cleanup;
use crate::device::Device;
use crate::error::Result;
use crate::image::Image;
use crate::types::*;

/// A
#[doc = crate::spec_link!("semaphore", "7", "synchronization-semaphores")]
pub struct Semaphore<'d> {
    pub(crate) signaller: Option<SemaphoreSignaller<'d>>,
    pub(crate) inner: Arc<SemaphoreRAII<'d>>,
}

#[derive(Debug)]
pub(crate) enum SemaphoreSignaller<'d> {
    Swapchain(Arc<Image<'d>>),
    Queue(Cleanup),
}

pub(crate) struct SemaphoreRAII<'d> {
    handle: Handle<VkSemaphore>,
    device: &'d Device<'d>,
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
        Ok(Self {
            signaller: None,
            inner: Arc::new(SemaphoreRAII { handle: handle.unwrap(), device }),
        })
    }
}

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

impl Drop for SemaphoreRAII<'_> {
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
        self.inner.handle.borrow()
    }
    /// Borrows the inner Vulkan handle.
    pub fn mut_handle(&mut self) -> Mut<VkSemaphore> {
        // Safe because the outer structure is mutably borrowed, and handle is
        // private.
        unsafe { self.inner.handle.borrow_mut_unchecked() }
    }

    /// Panics if there is no signaller
    pub(crate) fn take_signaller(&mut self) -> Arc<dyn Send + Sync + '_> {
        match self.signaller.take().unwrap() {
            SemaphoreSignaller::Queue(cleanup) => Arc::new(cleanup.raii()),
            SemaphoreSignaller::Swapchain(image) => image,
        }
    }
}
