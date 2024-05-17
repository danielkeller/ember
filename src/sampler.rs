// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::device::Device;
use crate::error::Result;
use crate::types::*;

/// A
#[doc = crate::spec_link!("sampler", "13", "samplers")]
#[derive(Debug, Eq)]
pub struct Sampler<'d> {
    handle: Handle<VkSampler>,
    device: &'d Device<'d>,
}

impl<'d> Sampler<'d> {
    #[doc = crate::man_link!(vkCreateSampler)]
    pub fn new(device: &'d Device, info: &SamplerCreateInfo) -> Result<Self> {
        device.increment_sampler_alloc_count()?;
        let mut handle = None;
        let result = unsafe {
            (device.fun.create_sampler)(
                device.handle(),
                info,
                None,
                &mut handle,
            )
        };
        if result.is_err() {
            device.decrement_sampler_alloc_count();
            result?
        }
        Ok(Self { handle: handle.unwrap(), device })
    }
}

impl Drop for Sampler<'_> {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun.destroy_sampler)(
                self.device.handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
        self.device.decrement_sampler_alloc_count();
    }
}

impl PartialEq for Sampler<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}

impl Sampler<'_> {
    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkSampler> {
        self.handle.borrow()
    }
    /// Returns the associated device.
    pub fn device(&self) -> &Device {
        self.device
    }
}
