use crate::device::Device;
use crate::enums::*;
use crate::error::Error;
use crate::error::Result;
// use crate::queue::Queue;
use crate::semaphore::Semaphore;
use crate::types::*;

use super::khr_surface::SurfaceResource;
use super::load::SwapchainDeviceFn;
use super::load::SwapchainKHRFn;
use super::SurfaceKHR;

pub struct KHRSwapchain {
    fun: SwapchainDeviceFn,
    device: Arc<Device>,
}

impl Device {
    pub fn khr_swapchain(self: &Arc<Self>) -> KHRSwapchain {
        KHRSwapchain {
            fun: SwapchainDeviceFn::new(self),
            device: self.clone(),
        }
    }
}

pub enum CreateSwapchainFrom {
    OldSwapchain(SwapchainKHR),
    Surface(SurfaceKHR),
}

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
    pub composite_alpha: CompositeAlphaFlagsKHR,
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

impl KHRSwapchain {
    pub fn create(
        &self,
        create_from: CreateSwapchainFrom,
        info: SwapchainCreateInfoKHR,
    ) -> Result<SwapchainKHR> {
        let (surface, mut old_swapchain) = match create_from {
            CreateSwapchainFrom::OldSwapchain(old) => {
                (old.surface, Some(old.handle))
            }
            CreateSwapchainFrom::Surface(surf) => (surf, None),
        };
        let mut handle = None;
        unsafe {
            (self.fun.create_swapchain_khr)(
                self.device.borrow(),
                &VkSwapchainCreateInfoKHR {
                    stype: Default::default(),
                    next: Default::default(),
                    flags: info.flags,
                    surface: surface.borrow(),
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
                        .map(|h| h.borrow_mut()),
                },
                None,
                &mut handle,
            )?;
        }
        let handle = handle.unwrap();
        let res = Arc::new(SwapchainImages {
            // Safety: Only used after the SwapchainKHR is destroyed.
            _handle: unsafe { handle.clone() },
            fun: SwapchainKHRFn::new(&self.device),
            device: self.device.clone(),
            _surface: surface.res.clone(),
        });
        Ok(SwapchainKHR { handle, res, surface })
    }
}

// Conceptually this owns the images, but it's also used to delay destruction
// of the swapchain until it's no longer used by the images.
pub struct SwapchainImages {
    /// Safety: Only use in Drop::drop
    _handle: Handle<VkSwapchainKHR>,
    fun: SwapchainKHRFn,
    device: Arc<Device>,
    // Needs to be destroyed after the swapchain
    _surface: Arc<SurfaceResource>,
}

impl Drop for SwapchainImages {
    fn drop(&mut self) {
        unsafe {
            (self.fun.destroy_swapchain_khr)(
                self.device.borrow(),
                self._handle.borrow_mut(),
                None,
            )
        }
    }
}

impl std::fmt::Debug for SwapchainImages {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SwapchainImages").finish()
    }
}

#[derive(Debug)]
pub struct SwapchainKHR {
    handle: Handle<VkSwapchainKHR>,
    res: Arc<SwapchainImages>,
    surface: SurfaceKHR,
}

// pub struct SwapchainImage {
//     //handle
//     res: Arc<SwapchainImages>,
//     index: u32,
// }

impl SwapchainKHR {
    pub fn borrow_mut(&mut self) -> Mut<'_, VkSwapchainKHR> {
        self.handle.borrow_mut()
    }
    pub fn images(&self) -> &Arc<SwapchainImages> {
        &self.res
    }
    pub fn images_mut(&mut self) -> &mut Arc<SwapchainImages> {
        &mut self.res
    }
    pub fn surface(&self) -> &SurfaceKHR {
        &self.surface
    }

    pub fn acquire_next_image(
        &mut self,
        signal: &mut Semaphore,
        timeout: u64,
    ) -> Result<(u32, bool)> {
        let mut index = 0;
        let res = unsafe {
            (self.res.fun.acquire_next_image_khr)(
                self.res.device.borrow(),
                self.handle.borrow_mut(),
                timeout,
                Some(signal.borrow_mut()),
                None,
                &mut index,
            )
        };
        match res {
            Ok(()) => Ok((index, false)),
            Err(e) => match e.into() {
                Error::SuboptimalHKR => Ok((index, true)),
                other => Err(other),
            },
        }
    }

    // pub fn present(
    //     &mut self,
    //     queue: &mut Queue,
    //     wait: &mut Semaphore,
    // ) -> Result<()> {
    //     Ok(())
    // }
}
