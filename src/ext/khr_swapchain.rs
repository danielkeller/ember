// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::intrinsics::transmute;
use std::mem::MaybeUninit;

use crate::device::Device;
use crate::enums::*;
use crate::error::{Error, Result};
use crate::ffi::ArrayMut;
use crate::image::Image;
use crate::queue::Queue;
use crate::semaphore::{Semaphore, SemaphoreSignaller};
use crate::subobject::{Owner, Subobject};
use crate::types::*;

use super::khr_surface::SurfaceKHR;

/// Whether to pass a previous swapchain to create the new one from.
pub enum CreateSwapchainFrom<'i> {
    OldSwapchain(SwapchainKHR<'i>),
    Surface(SurfaceKHR<'i>),
}

#[doc = crate::man_link!(VkSwapchainCreateInfoKHR)]
pub struct SwapchainCreateInfoKHR<'a> {
    pub flags: SwapchainCreateFlagsKHR,
    pub min_image_count: u32,
    pub image_format: Format,
    pub image_color_space: ColorSpaceKHR,
    pub image_extent: Extent2D,
    pub image_array_layers: u32,
    pub image_usage: ImageUsageFlags,
    pub image_sharing_mode: SharingMode,
    pub queue_family_indices: &'a [u32],
    pub pre_transform: SurfaceTransformKHR,
    pub composite_alpha: CompositeAlphaKHR,
    pub present_mode: PresentModeKHR,
    pub clipped: Bool,
}

impl<'a> Default for SwapchainCreateInfoKHR<'a> {
    fn default() -> Self {
        Self {
            flags: Default::default(),
            min_image_count: Default::default(),
            image_format: Default::default(),
            image_color_space: Default::default(),
            image_extent: Default::default(),
            image_array_layers: 1,
            image_usage: Default::default(),
            image_sharing_mode: Default::default(),
            queue_family_indices: Default::default(),
            pre_transform: Default::default(),
            composite_alpha: Default::default(),
            present_mode: Default::default(),
            clipped: Default::default(),
        }
    }
}

/// A
#[doc = crate::spec_link!("swapchain", "33", "_wsi_swapchain")]
#[derive(Debug)]
pub struct SwapchainKHR<'d> {
    images: Vec<(Arc<Image>, bool)>,
    res: Owner<SwapchainImages<'d>>,
    surface: SurfaceKHR<'d>,
}

// Conceptually this owns the images, but it's also used to delay destruction
// of the swapchain until it's no longer used by the images.
#[derive(Debug)]
pub(crate) struct SwapchainImages<'d> {
    handle: Handle<VkSwapchainKHR>,
    fun: SwapchainKHRFn,
    device: &'d Device<'d>,
    // _surface: Subobject<SurfaceLifetime<'d>>, // FIXME
}

impl<'d> SwapchainKHR<'d> {
    /// If `create_from` is [`CreateSwapchainFrom::OldSwapchain`], images in
    /// that swapchain that aren't acquired by the application are deleted. If
    /// any references remain to those images, returns
    /// [`Error::SynchronizationError`].
    /// Panics if the extension functions can't be loaded.
    ///
    #[doc = crate::man_link!(vkCreateSwapchainKHR)]
    pub fn new(
        device: &'d Device, create_from: CreateSwapchainFrom<'d>,
        info: SwapchainCreateInfoKHR,
    ) -> Result<Self> {
        let (mut surface, fun, mut old_swapchain) = match create_from {
            CreateSwapchainFrom::OldSwapchain(mut old) => {
                for (img, acquired) in &mut old.images {
                    if !*acquired && Arc::get_mut(img).is_none() {
                        return Err(Error::SynchronizationError);
                    }
                }
                (old.surface, old.res.fun.clone(), Some(old.res))
            }
            CreateSwapchainFrom::Surface(surf) => {
                (surf, SwapchainKHRFn::new(device), None)
            }
        };
        let mut handle = None;
        unsafe {
            (fun.create_swapchain_khr)(
                device.handle(),
                &VkSwapchainCreateInfoKHR {
                    stype: Default::default(),
                    next: Default::default(),
                    flags: info.flags,
                    surface: surface.mut_handle(),
                    min_image_count: info.min_image_count,
                    image_format: info.image_format,
                    image_color_space: info.image_color_space,
                    image_extent: info.image_extent,
                    image_array_layers: info.image_array_layers,
                    image_usage: info.image_usage,
                    image_sharing_mode: info.image_sharing_mode,
                    queue_family_indices: info.queue_family_indices.into(),
                    pre_transform: info.pre_transform,
                    composite_alpha: info.composite_alpha,
                    present_mode: info.present_mode,
                    clipped: info.clipped,
                    old_swapchain: old_swapchain
                        .as_mut()
                        .map(|h| h.handle.handle_mut()),
                },
                None,
                &mut handle,
            )?;
        }
        let handle = handle.unwrap();

        let mut n_images = 0;
        let mut images = vec![];
        unsafe {
            (fun.get_swapchain_images_khr)(
                device.handle(),
                handle.borrow(),
                &mut n_images,
                None,
            )?;
            images.reserve(n_images as usize);
            (fun.get_swapchain_images_khr)(
                device.handle(),
                handle.borrow(),
                &mut n_images,
                ArrayMut::from_slice(images.spare_capacity_mut()),
            )?;
            images.set_len(n_images as usize);
        }

        let res = Owner::new(SwapchainImages {
            handle,
            fun,
            device: device.clone(),
            // _surface: surface.resource(),
        });
        let images = images
            .into_iter()
            .map(|handle| {
                (
                    Arc::new(Image::new_from(
                        handle,
                        device.clone(),
                        Subobject::new(&res),
                        info.image_format,
                        info.image_extent.into(),
                        info.image_array_layers,
                        info.image_usage,
                    )),
                    false,
                )
            })
            .collect();

        Ok(Self { res, surface, images })
    }
}

impl Drop for SwapchainImages<'_> {
    fn drop(&mut self) {
        unsafe {
            (self.fun.destroy_swapchain_khr)(
                self.device.handle(),
                self.handle.handle_mut(),
                None,
            )
        }
    }
}

/// Whether the swapchain images are still optimal.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ImageOptimality {
    Optimal,
    Suboptimal,
}

impl<'i> SwapchainKHR<'i> {
    /// Borrows the inner Vulkan handle.
    pub fn mut_handle(&mut self) -> Mut<VkSwapchainKHR> {
        self.res.handle.handle_mut()
    }
    /// Returns the associated surface.
    pub fn surface(&self) -> &SurfaceKHR<'i> {
        &self.surface
    }

    /// Acquires the next swapchain image. [`Error::SuboptimalHKR`] is returned
    /// in the [`Ok`] variant.
    ///
    /// **Warning:** If `signal` is dropped without being waited on, it and the
    /// swapchain will be leaked.
    ///
    #[doc = crate::man_link!(vkAcquireNextImageKHR)]
    pub fn acquire_next_image(
        &mut self, signal: &mut Semaphore, timeout: u64,
    ) -> Result<(Arc<Image>, ImageOptimality)> {
        let mut index = 0;
        let res = &mut *self.res;
        let res = unsafe {
            (res.fun.acquire_next_image_khr)(
                res.device.handle(),
                res.handle.handle_mut(),
                timeout,
                Some(signal.mut_handle()),
                None,
                &mut index,
            )
        };
        let is_optimal = match res {
            Ok(()) => ImageOptimality::Optimal,
            Err(e) => match e.into() {
                Error::SuboptimalHKR => ImageOptimality::Suboptimal,
                other => return Err(other),
            },
        };
        let (image, acquired) = &mut self.images[index as usize];
        *acquired = true;
        let image = image.clone();
        signal.signaller = Some(SemaphoreSignaller::Swapchain(image.clone()));
        Ok((image, is_optimal))
    }

    /// Present the image. Returns [`Error::InvalidArgument`] if `wait` has no
    /// signal operation pending, or if the image did not come from this
    /// swapchain. The lifetime of the swapchain is also extended by the queue.
    pub fn present(
        &mut self, queue: &mut Queue, image: &Image, wait: &mut Semaphore,
    ) -> Result<ImageOptimality> {
        let index = self
            .images
            .iter()
            .position(|h| h.0.borrow() == image.borrow())
            .ok_or(Error::InvalidArgument)?;
        if wait.signaller.is_none() {
            return Err(Error::InvalidArgument);
        }

        let res = unsafe {
            (self.res.fun.queue_present_khr)(
                queue.mut_handle(),
                &PresentInfoKHR {
                    stype: Default::default(),
                    next: Default::default(),
                    wait: (&[wait.mut_handle()]).into(),
                    swapchains: (&[self.res.handle.handle_mut()]).into(),
                    indices: (&[index as u32]).into(),
                    results: None,
                },
            )
        };
        let is_optimal = match res {
            Ok(()) => ImageOptimality::Optimal,
            Err(e) => match e.into() {
                Error::SuboptimalHKR => ImageOptimality::Suboptimal,
                other => return Err(other),
            },
        };

        // Unacquire
        self.images[index].1 = false;
        // Semaphore signal op
        queue.add_resource(wait.take_signaller()); // Always needed?
        queue.add_resource(wait.inner.clone());
        // Actual present
        // queue.add_resource(Subobject::new(&self.res).erase()); // FIXME
        Ok(is_optimal)
    }
}

#[derive(Clone)]
pub struct SwapchainKHRFn {
    pub create_swapchain_khr: unsafe extern "system" fn(
        Ref<VkDevice>,
        &VkSwapchainCreateInfoKHR,
        Option<&'_ AllocationCallbacks>,
        &mut Option<Handle<VkSwapchainKHR>>,
    ) -> VkResult,
    pub destroy_swapchain_khr: unsafe extern "system" fn(
        Ref<VkDevice>,
        Mut<VkSwapchainKHR>,
        Option<&'_ AllocationCallbacks>,
    ),
    pub get_swapchain_images_khr: unsafe extern "system" fn(
        Ref<VkDevice>,
        Ref<VkSwapchainKHR>,
        &mut u32,
        Option<ArrayMut<MaybeUninit<Handle<VkImage>>>>,
    ) -> VkResult,
    pub acquire_next_image_khr: unsafe extern "system" fn(
        Ref<VkDevice>,
        Mut<VkSwapchainKHR>,
        u64,
        Option<Mut<VkSemaphore>>,
        Option<Mut<VkFence>>,
        &mut u32,
    ) -> VkResult,
    pub queue_present_khr: unsafe extern "system" fn(
        Mut<VkQueue>,
        &PresentInfoKHR<'_>,
    ) -> VkResult,
}

impl SwapchainKHRFn {
    pub fn new(dev: &Device) -> Self {
        unsafe {
            Self {
                create_swapchain_khr: transmute(
                    dev.get_proc_addr("vkCreateSwapchainKHR\0"),
                ),
                destroy_swapchain_khr: transmute(
                    dev.get_proc_addr("vkDestroySwapchainKHR\0"),
                ),
                get_swapchain_images_khr: transmute(
                    dev.get_proc_addr("vkGetSwapchainImagesKHR\0"),
                ),
                acquire_next_image_khr: transmute(
                    dev.get_proc_addr("vkAcquireNextImageKHR\0"),
                ),
                queue_present_khr: transmute(
                    dev.get_proc_addr("vkQueuePresentKHR\0"),
                ),
            }
        }
    }
}

impl std::fmt::Debug for SwapchainKHRFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SwapchainKHRFn").finish()
    }
}
