// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::device::Device;
use crate::types::*;

#[derive(Debug, Eq)]
struct SamplerInner {
    handle: Handle<VkSampler>,
    device: Device,
}

/// A
#[doc = crate::spec_link!("sampler", "13", "samplers")]
#[derive(Debug, PartialEq, Eq)]
pub struct Sampler {
    inner: Arc<SamplerInner>,
}

impl Sampler {
    #[doc = crate::man_link!(vkCreateSampler)]
    pub fn new(device: &Device, info: &SamplerCreateInfo) -> Self {
        device.increment_sampler_alloc_count();
        let mut handle = None;
        let result = unsafe {
            (device.fun().create_sampler)(
                device.handle(),
                info,
                None,
                &mut handle,
            )
        };
        if !result.is_success() {
            device.decrement_sampler_alloc_count();
            result.unwrap();
        }
        let inner = Arc::new(SamplerInner {
            handle: handle.unwrap(),
            device: device.clone(),
        });
        Self { inner }
    }
}

impl Drop for SamplerInner {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun().destroy_sampler)(
                self.device.handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
        self.device.decrement_sampler_alloc_count();
    }
}

impl PartialEq for SamplerInner {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}

impl Sampler {
    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkSampler> {
        self.inner.handle.borrow()
    }
    /// Returns the associated device.
    pub fn device(&self) -> &Device {
        &self.inner.device
    }

    pub(crate) fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}
