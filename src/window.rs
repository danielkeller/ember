// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Wrappers that integrate with [raw_window_handle](https://docs.rs/raw-window-handle/) and the platform's
#![doc = crate::spec_link!("wsi", "33", "wsi")]
//! extensions. This module is disabled by default because it requires
//! additional dependencies.

use std::mem::transmute;
use std::ptr::NonNull;

use crate::ext::{self, KHRXlibSurface};
use crate::ext::{KHRWaylandSurface, KHRWin32Surface, SurfaceKHR};
use crate::ffi::*;
use crate::instance::Instance;
use crate::physical_device::PhysicalDevice;
use crate::types::*;
use raw_window_handle::{
    HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle,
};

/// Return the required instance extensions to use WSI on the current platform.
pub fn required_instance_extensions(
    display: &impl HasDisplayHandle,
) -> &'static [Str<'static>] {
    match display.display_handle().unwrap().as_raw() {
        RawDisplayHandle::Windows(_) => {
            const WINDOWS_EXTS: [Str<'static>; 2] =
                [ext::SURFACE, ext::WIN32_SURFACE];
            &WINDOWS_EXTS
        }
        RawDisplayHandle::Wayland(_) => {
            const WAYLAND_EXTS: [Str<'static>; 2] =
                [ext::SURFACE, ext::WAYLAND_SURFACE];
            &WAYLAND_EXTS
        }
        RawDisplayHandle::Xlib(_) => {
            const XLIB_EXTS: [Str<'static>; 2] =
                [ext::SURFACE, ext::XLIB_SURFACE];
            &XLIB_EXTS
        }
        RawDisplayHandle::Xcb(_) => {
            const XCB_EXTS: [Str<'static>; 2] =
                [ext::SURFACE, ext::XCB_SURFACE];
            &XCB_EXTS
        }
        RawDisplayHandle::Android(_) => {
            const ANDROID_EXTS: [Str<'static>; 2] =
                [ext::SURFACE, ext::ANDROID_SURFACE];
            &ANDROID_EXTS
        }
        RawDisplayHandle::AppKit(_) => {
            const MACOS_EXTS: [Str<'static>; 2] =
                [ext::SURFACE, ext::METAL_SURFACE];
            &MACOS_EXTS
        }
        RawDisplayHandle::UiKit(_) => {
            const IOS_EXTS: [Str<'static>; 2] =
                [ext::SURFACE, ext::METAL_SURFACE];
            &IOS_EXTS
        }
        handle => panic!("Unsupported window handle type {handle:?}"),
    }
}

/// Returns true if the physical device and queue family index can present to
/// the display.
pub fn presentation_support(
    phy: &PhysicalDevice, queue_family_index: u32,
    display: &impl HasDisplayHandle,
) -> bool {
    match display.display_handle().unwrap().as_raw() {
        RawDisplayHandle::AppKit(_) => true,
        RawDisplayHandle::Xlib(_) => true,
        // winit doesn't set the visual_id for some reason so this doesn't work
        // unsafe {
        //     KHRXlibSurface::new(phy.instance()).presentation_support(
        //         phy,
        //         queue_family_index,
        //         NonNull::new(handle.display).unwrap(),
        //         handle.visual_id as usize,
        //     )
        // },
        RawDisplayHandle::Wayland(handle) => unsafe {
            KHRWaylandSurface::new(phy.instance()).presentation_support(
                phy,
                queue_family_index,
                handle.display,
            )
        },
        RawDisplayHandle::Windows(_) => KHRWin32Surface::new(phy.instance())
            .presentation_support(phy, queue_family_index),
        handle => panic!("Unsupported window handle type {handle:?}"),
    }
}

/// Create a surface for `window` with the appropriate extension for the current
/// platform.
pub fn create_surface<'i>(
    instance: &'i Instance, display: &impl HasDisplayHandle,
    window: &impl HasWindowHandle,
) -> SurfaceKHR {
    match (
        display.display_handle().unwrap().as_raw(),
        window.window_handle().unwrap().as_raw(),
    ) {
        #[cfg(any(target_os = "macos"))]
        (_, RawWindowHandle::AppKit(window)) => unsafe {
            use crate::ext::EXTMetalSurface;
            use raw_window_metal::{appkit, Layer};

            let layer = match appkit::metal_layer_from_handle(window) {
                Layer::Existing(layer) | Layer::Allocated(layer) => layer,
            };
            EXTMetalSurface::new(instance).create_metal_surface_ext(
                &MetalSurfaceCreateInfoEXT {
                    stype: Default::default(),
                    next: Default::default(),
                    flags: Default::default(),
                    layer: NonNull::new(layer as *mut c_void).unwrap(),
                },
            )
        },
        (RawDisplayHandle::Xlib(display), RawWindowHandle::Xlib(window)) => unsafe {
            KHRXlibSurface::new(instance).create_xlib_surface_ext(
                &XlibSurfaceCreateInfoKHR {
                    stype: Default::default(),
                    next: Default::default(),
                    flags: Default::default(),
                    display: display.display.unwrap(),
                    window: window.window as usize,
                },
            )
        },
        (
            RawDisplayHandle::Wayland(display),
            RawWindowHandle::Wayland(window),
        ) => unsafe {
            KHRWaylandSurface::new(instance).create_wayland_surface_ext(
                &WaylandSurfaceCreateInfoKHR {
                    stype: Default::default(),
                    next: Default::default(),
                    flags: Default::default(),
                    display: display.display,
                    surface: window.surface,
                },
            )
        },
        (_, RawWindowHandle::Win32(window)) => unsafe {
            let hinstance: Option<NonNull<c_void>> =
                transmute(window.hinstance);
            let hwnd: NonNull<c_void> = transmute(window.hwnd);

            KHRWin32Surface::new(instance).create_win32_surface_ext(
                &Win32SurfaceCreateInfoKHR {
                    stype: Default::default(),
                    next: Default::default(),
                    flags: Default::default(),
                    hinstance: hinstance.unwrap(),
                    hwnd,
                },
            )
        },
        dw => panic!("Unsupported display and window combination {dw:?}"),
    }
}
