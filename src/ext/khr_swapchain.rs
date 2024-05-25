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
use crate::semaphore::Semaphore;
use crate::types::*;

use super::khr_surface::SurfaceKHR;

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
pub struct SwapchainKHR<'i, 'd> {
    handle: Handle<VkSwapchainKHR>,
    fun: SwapchainKHRFn,
    images: Vec<Image<'d>>, // Warning: This lifetime is a lie...
    // Is it ok for 'a to be invariant?
    surface: Option<&'d mut SurfaceKHR<'i>>,
    device: &'d Device<'d>,
}

// Maybe it would be more conveient to just make Swapchain !Sync.
pub struct SwapchainImages<'chain> {
    handle: Mut<'chain, VkSwapchainKHR>,
    fun: &'chain SwapchainKHRFn,
    device: &'chain Device<'chain>,
    images: &'chain [Image<'chain>], // ... this is the real one.
}

impl<'i, 'd> SwapchainKHR<'i, 'd> {
    /// Panics if the extension functions can't be loaded.
    ///
    #[doc = crate::man_link!(vkCreateSwapchainKHR)]
    pub fn new(
        device: &'d Device, surface: &'d mut SurfaceKHR<'i>,
        info: SwapchainCreateInfoKHR,
    ) -> Result<Self> {
        Self::new_impl(device, surface, SwapchainKHRFn::new(device), None, info)
    }

    /// The current swapchain is destroyed after the new one is created.
    ///
    #[doc = crate::man_link!(vkCreateSwapchainKHR)]
    pub fn recreate(&mut self, info: SwapchainCreateInfoKHR) -> Result<()> {
        // I think this puts 'self' in a bad state if this fails...
        let mut new = Self::new_impl(
            self.device,
            self.surface.take().unwrap(),
            SwapchainKHRFn::new(self.device),
            Some(self.handle.borrow_mut()),
            info,
        )?;
        std::mem::swap(self, &mut new);
        Ok(())
    }

    fn new_impl(
        device: &'d Device, surface: &'d mut SurfaceKHR<'i>,
        fun: SwapchainKHRFn, old_swapchain: Option<Mut<'_, VkSwapchainKHR>>,
        info: SwapchainCreateInfoKHR,
    ) -> Result<Self> {
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
                    old_swapchain,
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

        let images = unsafe {
            images
                .into_iter()
                .map(|handle| {
                    Image::new_from(
                        handle,
                        device,
                        info.image_format,
                        info.image_extent.into(),
                        info.image_array_layers,
                        info.image_usage,
                    )
                })
                .collect()
        };

        Ok(Self { device, handle, fun, images, surface: Some(surface) })
    }

    pub fn images(&mut self) -> SwapchainImages {
        SwapchainImages {
            handle: self.handle.borrow_mut(),
            fun: &self.fun,
            device: self.device,
            images: &self.images,
        }
    }
}

impl Drop for SwapchainKHR<'_, '_> {
    fn drop(&mut self) {
        unsafe {
            (self.fun.destroy_swapchain_khr)(
                self.device.handle(),
                self.handle.borrow_mut(),
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

impl<'chain> SwapchainImages<'chain> {
    /// Borrows the inner Vulkan handle.
    pub fn mut_handle(&mut self) -> Mut<VkSwapchainKHR> {
        self.handle.reborrow_mut()
    }

    pub fn images(&self) -> &'chain [Image<'chain>] {
        &self.images
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
    ) -> Result<(usize, ImageOptimality)> {
        let mut index = 0;
        let res = unsafe {
            (self.fun.acquire_next_image_khr)(
                self.device.handle(),
                self.handle.reborrow_mut(),
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
        // ???
        // signal.signaller = Some(SemaphoreSignaller::Swapchain(image.clone()));
        Ok((index as usize, is_optimal))
    }

    /// Present the image. Returns [`Error::InvalidArgument`] if `wait` has no
    /// signal operation pending, or if the image did not come from this
    /// swapchain. The lifetime of the swapchain is also extended by the queue.
    #[doc = crate::man_link!(vkQueuePresentKHR)]
    pub fn present(
        &mut self, queue: &mut Queue, image: usize, wait: &mut Semaphore,
    ) -> Result<ImageOptimality> {
        assert!(image < self.images.len(), "'image' out of bounds");
        if wait.signaller.is_none() {
            return Err(Error::InvalidArgument);
        }

        let res = unsafe {
            (self.fun.queue_present_khr)(
                queue.mut_handle(),
                &PresentInfoKHR {
                    stype: Default::default(),
                    next: Default::default(),
                    wait: (&[wait.mut_handle()]).into(),
                    swapchains: (&[self.handle.reborrow_mut()]).into(),
                    indices: (&[image as u32]).into(),
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

        // ???
        // Semaphore signal op
        // queue.add_resource(wait.take_signaller()); // Always needed?
        // queue.add_resource(wait.inner.clone());
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
