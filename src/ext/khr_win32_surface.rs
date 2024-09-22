// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::intrinsics::transmute;

use crate::enums::*;
use crate::types::*;

use crate::error::VkResult;
use crate::instance::Instance;
use crate::physical_device::PhysicalDevice;

use super::khr_surface::SurfaceKHR;

/// An KHR_win32_surface extension object.
pub struct KHRWin32Surface<'i> {
    fun: Win32SurfaceFn,
    instance: &'i Instance,
}

impl<'i> KHRWin32Surface<'i> {
    /// Creates an [`KHRWin32Surface`] extension object. Panics if the extension
    /// functions can't be loaded.
    pub fn new(instance: &'i Instance) -> Self {
        Self { fun: Win32SurfaceFn::new(instance), instance }
    }

    #[doc = crate::man_link!(vkGetPhysicalDeviceWin32PresentationSupportKHR)]
    pub fn presentation_support(
        &self, phy: &PhysicalDevice, queue_family_index: u32,
    ) -> bool {
        unsafe {
            (self.fun.get_physical_device_win32_presentation_support_khr)(
                phy.handle(),
                queue_family_index,
            )
        }
        .into()
    }
    #[doc = crate::man_link!(vkCreateWin32SurfaceKHR)]
    pub unsafe fn create_win32_surface_ext(
        &self, info: &Win32SurfaceCreateInfoKHR,
    ) -> SurfaceKHR {
        let mut handle = None;
        (self.fun.create_win32_surface_khr)(
            self.instance.handle(),
            info,
            None,
            &mut handle,
        )
        .unwrap();
        SurfaceKHR::new(handle.unwrap(), self.instance)
    }
}

#[non_exhaustive]
pub struct Win32SurfaceFn {
    pub get_physical_device_win32_presentation_support_khr:
        unsafe extern "system" fn(Ref<VkPhysicalDevice>, u32) -> Bool,
    pub create_win32_surface_khr: unsafe extern "system" fn(
        Ref<VkInstance>,
        &Win32SurfaceCreateInfoKHR,
        Option<&'_ AllocationCallbacks>,
        &mut Option<Handle<VkSurfaceKHR>>,
    ) -> VkResult,
}

impl Win32SurfaceFn {
    pub fn new(inst: &Instance) -> Self {
        Self {
            get_physical_device_win32_presentation_support_khr: unsafe {
                transmute(inst.get_proc_addr(
                    "vkGetPhysicalDeviceWin32PresentationSupportKHR\0",
                ))
            },
            create_win32_surface_khr: unsafe {
                transmute(inst.get_proc_addr("vkCreateWin32SurfaceKHR\0"))
            },
        }
    }
}
