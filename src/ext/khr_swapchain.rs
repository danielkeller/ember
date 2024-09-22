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
use crate::error::{OutOfDeviceMemory, VkResult};
use crate::ffi::ArrayMut;
use crate::image::Image;
use crate::queue::{Queue, SubmitScope};
use crate::semaphore::Semaphore;
use crate::subobject::Owner;
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

#[derive(Debug)]
pub(crate) struct SwapchainInner {
    handle: Handle<VkSwapchainKHR>,
    device: Device,
    fun: SwapchainKHRFn,
}

/// A
#[doc = crate::spec_link!("swapchain", "33", "_wsi_swapchain")]
#[derive(Debug)]
pub struct SwapchainKHR {
    inner: Owner<SwapchainInner>,
    images: Vec<Image>,
    image_acquired: Vec<bool>,
    surface: SurfaceKHR,
}

impl SwapchainKHR {
    /// Panics if the extension functions can't be loaded.
    ///
    #[doc = crate::man_link!(vkCreateSwapchainKHR)]
    pub fn new(
        device: &Device, surface: SurfaceKHR, info: &SwapchainCreateInfoKHR,
    ) -> Self {
        Self::try_new(device, surface, info).unwrap()
    }
    /// Panics if the extension functions can't be loaded.
    ///
    #[doc = crate::man_link!(vkCreateSwapchainKHR)]
    pub fn try_new(
        device: &Device, surface: SurfaceKHR, info: &SwapchainCreateInfoKHR,
    ) -> Result<Self, OutOfDeviceMemory> {
        let fun = SwapchainKHRFn::new(device);
        Self::create(device, surface, fun, info, None)
    }

    /// [ImageView]s, [Framebuffer]s, and so on which refer to images that are
    /// not acquired must be dropped before calling this function, or it will
    /// panic.
    ///
    #[doc = crate::man_link!(vkCreateSwapchainKHR)]
    pub fn recreate(mut self, info: &SwapchainCreateInfoKHR) -> Self {
        Self::try_recreate(self, info).unwrap()
    }

    /// [ImageView]s, [Framebuffer]s, and so on which refer to images that are
    /// not acquired must be dropped before calling this function, or it will
    /// panic.
    ///
    #[doc = crate::man_link!(vkCreateSwapchainKHR)]
    pub fn try_recreate(
        mut self, info: &SwapchainCreateInfoKHR,
    ) -> Result<Self, OutOfDeviceMemory> {
        let inner = &mut *self.inner;
        for (i, image) in self.images.iter_mut().enumerate() {
            if !self.image_acquired[i]
                && Arc::get_mut(&mut image.inner).is_none()
            {
                panic!(
                    "Cannot recreate swapchain until all references to \
                    non-acquired images are dropped."
                )
            }
        }
        Self::create(
            &inner.device,
            self.surface,
            inner.fun.clone(),
            info,
            Some(inner.handle.borrow_mut()),
        )
    }

    fn create(
        device: &Device, mut surface: SurfaceKHR, fun: SwapchainKHRFn,
        info: &SwapchainCreateInfoKHR,
        old_swapchain: Option<Mut<VkSwapchainKHR>>,
    ) -> Result<Self, OutOfDeviceMemory> {
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
            )
            .unwrap_or_oom()?;
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
            )
            .unwrap();
            images.reserve(n_images as usize);
            (fun.get_swapchain_images_khr)(
                device.handle(),
                handle.borrow(),
                &mut n_images,
                ArrayMut::from_slice(images.spare_capacity_mut()),
            )
            .unwrap();
            images.set_len(n_images as usize);
        }

        let inner =
            Owner::new(SwapchainInner { handle, device: device.clone(), fun });

        let images = images
            .into_iter()
            .map(|handle| {
                Image::new_from_swapchain(
                    handle,
                    device.clone(),
                    info.image_format,
                    info.image_extent.into(),
                    info.image_array_layers,
                    info.image_usage,
                    inner.downgrade(),
                )
            })
            .collect::<Vec<_>>();

        Ok(SwapchainKHR {
            inner,
            image_acquired: vec![false; images.len()],
            images,
            surface,
        })
    }

    /// Drops the swapchain and returns the surface it was associated with. This
    /// function still works even if the devie was lost. Note that the swapchain
    /// will not be actually destroyed until the images that refer to it are
    /// dropped, and attempting to associate the surface with another swapchain
    /// before that happens will fail.
    pub fn into_surface(self) -> SurfaceKHR {
        self.surface
    }

    /// Borrows the inner Vulkan handle.
    pub fn mut_handle(&mut self) -> Mut<VkSwapchainKHR> {
        self.inner.handle.borrow_mut()
    }

    pub fn images(&self) -> &[Image] {
        &self.images
    }
}

impl Drop for SwapchainInner {
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

impl<'scope> SubmitScope<'scope> {
    /// Acquire the next image from the swapchain, and queue a semaphore wait
    /// on the acquire.
    pub fn acquire_next_image(
        &mut self, swapchain: &mut SwapchainKHR, timeout: u64,
        semaphore: &'scope mut Semaphore, wait_mask: PipelineStageFlags,
    ) -> Result<(usize, ImageOptimality), AcquireError> {
        let inner = &mut *swapchain.inner;
        let mut index = 0;
        let res = unsafe {
            (inner.fun.acquire_next_image_khr)(
                inner.device.handle(),
                inner.handle.borrow_mut(),
                timeout,
                Some(semaphore.mut_handle()),
                None,
                &mut index,
            )
        };
        self.wait(semaphore, wait_mask);
        res.unwrap_or_acquire_error()?;
        let is_optimal = if res == VkResult::SUBOPTIMAL_HKR {
            ImageOptimality::Suboptimal
        } else {
            ImageOptimality::Optimal
        };
        Ok((index as usize, is_optimal))
    }

    /// Present the image using the semaphore. The signal must be submitted
    /// already.
    pub fn present(
        &mut self, swapchain: &mut SwapchainKHR, image: usize,
        semaphore: &mut Semaphore,
    ) -> ImageOptimality {
        assert!(image < swapchain.images.len(), "'image' out of bounds");
        let inner = &mut *swapchain.inner;

        let res = unsafe {
            (inner.fun.queue_present_khr)(
                self.mut_handle(),
                &PresentInfoKHR {
                    stype: Default::default(),
                    next: Default::default(),
                    wait: (&[semaphore.mut_handle()]).into(),
                    swapchains: (&[inner.handle.borrow_mut()]).into(),
                    indices: (&[image as u32]).into(),
                    results: None,
                },
            )
        };
        res.unwrap();
        if res == VkResult::SUBOPTIMAL_HKR {
            ImageOptimality::Suboptimal
        } else {
            ImageOptimality::Optimal
        }
    }
}

#[cfg(any())]
impl<'chain> SwapchainImages<'chain> {
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

        Ok(is_optimal)
    }
}

#[derive(Clone)]
#[non_exhaustive]
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct OutOfDateKHR;

impl std::fmt::Display for OutOfDateKHR {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Swapchain out of date")
    }
}
impl std::error::Error for OutOfDateKHR {}

impl VkResult {
    #[track_caller]
    pub fn unwrap_or_out_of_date(self) -> Result<(), OutOfDateKHR> {
        match self {
            Self::OUT_OF_DATE_KHR => Err(OutOfDateKHR),
            other => Ok(other.unwrap()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcquireError {
    OutOfDate,
    Timeout,
    NotReady,
}

impl std::fmt::Display for AcquireError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AcquireError::OutOfDate => f.write_str("Swapchain out of date"),
            AcquireError::Timeout => f.write_str("Timout"),
            AcquireError::NotReady => f.write_str("Not ready"),
        }
    }
}
impl std::error::Error for AcquireError {}

impl VkResult {
    #[track_caller]
    pub fn unwrap_or_acquire_error(self) -> Result<(), AcquireError> {
        match self {
            Self::OUT_OF_DATE_KHR => Err(AcquireError::OutOfDate),
            Self::TIMEOUT => Err(AcquireError::Timeout),
            Self::NOT_READY => Err(AcquireError::NotReady),
            other => Ok(other.unwrap()),
        }
    }
}
