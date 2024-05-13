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

use std::ptr::NonNull;

use crate::error::{Error, Result};
use crate::ext::{self, KHRXlibSurface};
use crate::ext::{KHRWaylandSurface, KHRWin32Surface, SurfaceKHR};
use crate::ffi::*;
use crate::instance::Instance;
use crate::physical_device::PhysicalDevice;
use crate::types::*;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

/// Return the required instance extensions to use WSI on the current platform.
pub fn required_instance_extensions(
    window: &impl HasRawWindowHandle,
) -> Result<&'static [Str<'static>]> {
    match window.raw_window_handle() {
        RawWindowHandle::Win32(_) => {
            const WINDOWS_EXTS: [Str<'static>; 2] =
                [ext::SURFACE, ext::WIN32_SURFACE];
            Ok(&WINDOWS_EXTS)
        }
        RawWindowHandle::Wayland(_) => {
            const WAYLAND_EXTS: [Str<'static>; 2] =
                [ext::SURFACE, ext::WAYLAND_SURFACE];
            Ok(&WAYLAND_EXTS)
        }
        RawWindowHandle::Xlib(_) => {
            const XLIB_EXTS: [Str<'static>; 2] =
                [ext::SURFACE, ext::XLIB_SURFACE];
            Ok(&XLIB_EXTS)
        }
        RawWindowHandle::Xcb(_) => {
            const XCB_EXTS: [Str<'static>; 2] =
                [ext::SURFACE, ext::XCB_SURFACE];
            Ok(&XCB_EXTS)
        }
        RawWindowHandle::AndroidNdk(_) => {
            const ANDROID_EXTS: [Str<'static>; 2] =
                [ext::SURFACE, ext::ANDROID_SURFACE];
            Ok(&ANDROID_EXTS)
        }
        RawWindowHandle::AppKit(_) => {
            const MACOS_EXTS: [Str<'static>; 2] =
                [ext::SURFACE, ext::METAL_SURFACE];
            Ok(&MACOS_EXTS)
        }
        RawWindowHandle::UiKit(_) => {
            const IOS_EXTS: [Str<'static>; 2] =
                [ext::SURFACE, ext::METAL_SURFACE];
            Ok(&IOS_EXTS)
        }
        _ => Err(Error::ExtensionNotPresent),
    }
}

/// Returns true if the physical device and queue family index can present to
/// the window.
pub fn presentation_support(
    phy: &PhysicalDevice, queue_family_index: u32,
    window: &impl HasRawWindowHandle,
) -> bool {
    match window.raw_window_handle() {
        RawWindowHandle::AppKit(_) => true,
        RawWindowHandle::Xlib(_) => true,
        // winit doesn't set the visual_id for some reason so this doesn't work
        // unsafe {
        //     KHRXlibSurface::new(phy.instance()).presentation_support(
        //         phy,
        //         queue_family_index,
        //         NonNull::new(handle.display).unwrap(),
        //         handle.visual_id as usize,
        //     )
        // },
        RawWindowHandle::Wayland(handle) => unsafe {
            KHRWaylandSurface::new(phy.instance()).presentation_support(
                phy,
                queue_family_index,
                NonNull::new(handle.display).unwrap(),
            )
        },
        RawWindowHandle::Win32(_) => KHRWin32Surface::new(phy.instance())
            .presentation_support(phy, queue_family_index),
        handle => panic!("Unimplemented window handle type: {:?}", handle),
    }
}

/// Create a surface for `window` with the appropriate extension for the current
/// platform.
pub fn create_surface<'i>(
    instance: &'i Instance, window: &impl HasRawWindowHandle,
) -> Result<SurfaceKHR<'i>> {
    match window.raw_window_handle() {
        #[cfg(any(target_os = "macos"))]
        RawWindowHandle::AppKit(handle) => {
            use crate::ext::EXTMetalSurface;
            use raw_window_metal::{appkit, Layer};

            unsafe {
                match appkit::metal_layer_from_handle(handle) {
                    Layer::Existing(layer) | Layer::Allocated(layer) => {
                        EXTMetalSurface::new(instance).create_metal_surface_ext(
                            &MetalSurfaceCreateInfoEXT {
                                stype: Default::default(),
                                next: Default::default(),
                                flags: Default::default(),
                                layer: NonNull::new(layer as *mut c_void)
                                    .unwrap(),
                            },
                        )
                    }
                    Layer::None => Err(Error::Other), //TODO
                }
            }
        }
        RawWindowHandle::Xlib(handle) => unsafe {
            KHRXlibSurface::new(instance).create_xlib_surface_ext(
                &XlibSurfaceCreateInfoKHR {
                    stype: Default::default(),
                    next: Default::default(),
                    flags: Default::default(),
                    display: NonNull::new(handle.display).unwrap(),
                    window: handle.window as usize,
                },
            )
        },
        RawWindowHandle::Wayland(handle) => unsafe {
            KHRWaylandSurface::new(instance).create_wayland_surface_ext(
                &WaylandSurfaceCreateInfoKHR {
                    stype: Default::default(),
                    next: Default::default(),
                    flags: Default::default(),
                    display: NonNull::new(handle.display).unwrap(),
                    surface: NonNull::new(handle.surface).unwrap(),
                },
            )
        },
        RawWindowHandle::Win32(handle) => unsafe {
            KHRWin32Surface::new(instance).create_win32_surface_ext(
                &Win32SurfaceCreateInfoKHR {
                    stype: Default::default(),
                    next: Default::default(),
                    flags: Default::default(),
                    hinstance: NonNull::new(handle.hinstance).unwrap(),
                    hwnd: NonNull::new(handle.hwnd).unwrap(),
                },
            )
        },
        _ => Err(Error::ExtensionNotPresent),
    }
}
