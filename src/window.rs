use std::ptr::NonNull;
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::ext;
use crate::ext::SurfaceKHR;
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
    phy: &PhysicalDevice,
    queue_family_index: u32,
    window: &impl HasRawWindowHandle,
) -> bool {
    match window.raw_window_handle() {
        RawWindowHandle::AppKit(_) => true,
        RawWindowHandle::Xlib(handle) => unsafe {
            phy.instance().khr_xlib_surface().presentation_support(
                phy,
                queue_family_index,
                NonNull::new(handle.display).unwrap(),
                handle.visual_id as usize,
            )
        },
        _ => false,
    }
}

/// Create a surface for 'window' with the appropriate extension for the current
/// platform.
pub fn create_surface(
    instance: &Arc<Instance>,
    window: &impl HasRawWindowHandle,
) -> Result<SurfaceKHR> {
    match window.raw_window_handle() {
        #[cfg(any(target_os = "macos"))]
        RawWindowHandle::AppKit(handle) => {
            use raw_window_metal::{appkit, Layer};

            unsafe {
                match appkit::metal_layer_from_handle(handle) {
                    Layer::Existing(layer) | Layer::Allocated(layer) => {
                        instance.ext_metal_surface().create_metal_surface_ext(
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
            instance.khr_xlib_surface().create_xlib_surface_ext(
                &XlibSurfaceCreateInfoKHR {
                    stype: Default::default(),
                    next: Default::default(),
                    flags: Default::default(),
                    display: NonNull::new(handle.display).unwrap(),
                    window: handle.window as usize,
                },
            )
        },
        _ => Err(Error::ExtensionNotPresent),
    }
}
