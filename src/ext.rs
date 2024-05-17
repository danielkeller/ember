// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Vulkan extensions

use crate::ffi::Str;

mod ext_metal_surface;
mod khr_surface;
mod khr_xlib_surface;
mod khr_wayland_surface;
mod khr_win32_surface;
pub(crate) mod khr_swapchain;

pub use ext_metal_surface::EXTMetalSurface;
pub use khr_surface::SurfaceKHR;
pub use khr_swapchain::{SwapchainCreateInfoKHR, SwapchainKHR};
pub use khr_wayland_surface::KHRWaylandSurface;
pub use khr_win32_surface::KHRWin32Surface;
pub use khr_xlib_surface::KHRXlibSurface;

// Instance level extensions

/// VK_KHR_surface instance extension name
pub const SURFACE: Str<'static> =
    unsafe { Str::new_unchecked(b"VK_KHR_surface\0") };
/// VK_KHR_win32_surface instance extension name
pub const WIN32_SURFACE: Str<'static> =
    unsafe { Str::new_unchecked(b"VK_KHR_win32_surface\0") };
/// VK_KHR_wayland_surface instance extension name
pub const WAYLAND_SURFACE: Str<'static> =
    unsafe { Str::new_unchecked(b"VK_KHR_wayland_surface\0") };
/// VK_KHR_xlib_surface instance extension name
pub const XLIB_SURFACE: Str<'static> =
    unsafe { Str::new_unchecked(b"VK_KHR_xlib_surface\0") };
/// VK_KHR_xcb_surface instance extension name
pub const XCB_SURFACE: Str<'static> =
    unsafe { Str::new_unchecked(b"VK_KHR_xcb_surface\0") };
/// VK_KHR_android_surface instance extension name
pub const ANDROID_SURFACE: Str<'static> =
    unsafe { Str::new_unchecked(b"VK_KHR_android_surface\0") };
/// VK_EXT_metal_surface instance extension name
pub const METAL_SURFACE: Str<'static> =
    unsafe { Str::new_unchecked(b"VK_EXT_metal_surface\0") };
/// VK_KHR_get_physical_device_properties2 instance extension name
pub const GET_PHYSICAL_DEVICE_PROPERTIES2: Str<'static> =
    unsafe { Str::new_unchecked(b"VK_KHR_get_physical_device_properties2\0") };
/// VK_KHR_portability_enumeration instance extension name
pub const PORTABILITY_ENUMERATION: Str<'static> =
    unsafe { Str::new_unchecked(b"VK_KHR_portability_enumeration\0") };

// Device level extensions

/// VK_KHR_portability_subset device extension name
pub const PORTABILITY_SUBSET: Str<'static> =
    unsafe { Str::new_unchecked(b"VK_KHR_portability_subset\0") };
/// VK_KHR_swapchain device extension name
pub const SWAPCHAIN: Str<'static> =
    unsafe { Str::new_unchecked(b"VK_KHR_swapchain\0") };
