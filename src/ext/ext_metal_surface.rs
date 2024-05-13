// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::intrinsics::transmute;

use crate::types::*;

use crate::error::Result;
use crate::instance::Instance;

use super::khr_surface::SurfaceKHR;

/// An EXT_metal_surface extension object.
pub struct EXTMetalSurface<'i> {
    fun: MetalSurfaceFn,
    instance: &'i Instance,
}

impl<'i> EXTMetalSurface<'i> {
    /// Creates an [`EXTMetalSurface`] extension object. Panics if the extension
    /// functions can't be loaded.
    pub fn new(instance: &'i Instance) -> Self {
        Self { fun: MetalSurfaceFn::new(instance), instance }
    }

    /// Creates a metal surface. The `layer` member of
    /// [`MetalSurfaceCreateInfoEXT`] must refer to a valid Metal layer.
    #[doc = crate::man_link!(vkCreateMetalSurfaceEXT)]
    pub unsafe fn create_metal_surface_ext(
        &self, info: &MetalSurfaceCreateInfoEXT,
    ) -> Result<SurfaceKHR<'i>> {
        let mut handle = None;
        (self.fun.create_metal_surface_ext)(
            self.instance.handle(),
            info,
            None,
            &mut handle,
        )?;
        Ok(SurfaceKHR::new(handle.unwrap(), self.instance))
    }
}

pub struct MetalSurfaceFn {
    pub create_metal_surface_ext: unsafe extern "system" fn(
        Ref<VkInstance>,
        &MetalSurfaceCreateInfoEXT,
        Option<&'_ AllocationCallbacks>,
        &mut Option<Handle<VkSurfaceKHR>>,
    ) -> VkResult,
}

impl MetalSurfaceFn {
    pub fn new(inst: &Instance) -> Self {
        Self {
            create_metal_surface_ext: unsafe {
                transmute(inst.get_proc_addr("vkCreateMetalSurfaceEXT\0"))
            },
        }
    }
}
