// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::device::Device;
use crate::error::{Error, Result};
use crate::types::*;

/// A
#[doc = crate::spec_link!("shader module", "shaders")]
pub struct ShaderModule {
    handle: Handle<VkShaderModule>,
    device: Arc<Device>,
}

impl ShaderModule {
    #[doc = crate::man_link!(vkCreateShaderModule)]
    pub fn new(device: &Arc<Device>, code: &[u32]) -> Result<Self> {
        if code.is_empty() {
            return Err(Error::InvalidArgument);
        }
        let mut handle = None;
        unsafe {
            (device.fun.create_shader_module)(
                device.handle(),
                &VkShaderModuleCreateInfo {
                    stype: Default::default(),
                    next: Default::default(),
                    flags: Default::default(),
                    code: code.into(),
                },
                None,
                &mut handle,
            )?;
        }
        Ok(Self { handle: handle.unwrap(), device: device.clone() })
    }
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun.destroy_shader_module)(
                self.device.handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

impl ShaderModule {
    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkShaderModule> {
        self.handle.borrow()
    }
}
