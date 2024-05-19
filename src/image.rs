// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::enums::*;
use crate::error::{Error, Result};
use crate::memory::DeviceMemory;
use crate::types::*;
use crate::vk::Device;

use std::fmt::Debug;

/// An image with no memory. Call [`Image::new`] to bind memory and create an
/// [`Image`].
#[derive(Debug)]
pub struct ImageWithoutMemory<'d> {
    handle: Handle<VkImage>,
    format: Format,
    extent: Extent3D,
    mip_levels: u32,
    array_layers: u32,
    usage: ImageUsageFlags,
    owned: bool,
    device: &'d Device<'d>,
}

/// An
#[doc = crate::spec_link!("image", "12", "resources-images")]
/// with memory attached to it.
#[derive(Debug)]
pub struct Image<'a>(ImageWithoutMemory<'a>);

impl<'a> std::ops::Deref for Image<'a> {
    type Target = ImageWithoutMemory<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'d> ImageWithoutMemory<'d> {
    #[doc = crate::man_link!(vkCreateImage)]
    pub fn new(device: &'d Device, info: &ImageCreateInfo<'_>) -> Result<Self> {
        let max_dim =
            info.extent.width.max(info.extent.height).max(info.extent.depth);
        if (info.image_type == ImageType::_1D
            && max_dim > device.limits().max_image_dimension_1d)
            || (info.image_type == ImageType::_2D
                && (info.flags & ImageCreateFlags::CUBE_COMPATIBLE).is_empty()
                && max_dim > device.limits().max_image_dimension_2d)
            || (info.image_type == ImageType::_2D
                && !(info.flags & ImageCreateFlags::CUBE_COMPATIBLE).is_empty()
                && max_dim > device.limits().max_image_dimension_cube)
            || (info.image_type == ImageType::_3D
                && max_dim > device.limits().max_image_dimension_3d)
            || (info.array_layers > device.limits().max_image_array_layers)
        {
            return Err(Error::LimitExceeded);
        }
        let mut handle = None;
        unsafe {
            (device.fun.create_image)(
                device.handle(),
                info,
                None,
                &mut handle,
            )?;
        }
        Ok(Self {
            handle: handle.unwrap(),
            extent: info.extent,
            format: info.format,
            mip_levels: info.mip_levels,
            array_layers: info.array_layers,
            usage: info.usage,
            owned: true,
            device,
        })
    }
    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkImage> {
        self.handle.borrow()
    }
    /// Borrows the inner Vulkan handle.
    pub fn handle_mut(&mut self) -> Mut<VkImage> {
        self.handle.borrow_mut()
    }
    /// Returns the associated device.
    pub fn device(&self) -> &Device {
        self.device
    }
    /// If [`ImageCreateInfo::usage`] includes a storage image usage type and
    /// the robust buffer access feature was not enabled at device creation, any
    /// host-visible memory types will be removed from the output. Note that on
    /// some physical devices (eg software rasterizers), *all* memory types are
    /// host-visible.
    ///
    #[doc = crate::man_link!(vkGetImageMemoryRequirements)]
    pub fn memory_requirements(&self) -> MemoryRequirements {
        let mut result = Default::default();
        unsafe {
            (self.device.fun.get_image_memory_requirements)(
                self.device.handle(),
                self.handle.borrow(),
                &mut result,
            );
        }
        if !self.device.enabled().robust_buffer_access.as_bool()
            && self.usage.indexable()
        {
            result.clear_host_visible_types(
                &self.device.physical_device().memory_properties(),
            );
        }
        result
    }
    /// Returns the allowed image usages
    pub fn usage(&self) -> ImageUsageFlags {
        self.usage
    }
    /// Returns the format of the image.
    pub fn format(&self) -> Format {
        self.format
    }
    /// Returns the extent of the image.
    pub fn extent(&self, mip_level: u32) -> Extent3D {
        let ex = self.extent;
        Extent3D {
            width: ex.width >> mip_level,
            height: ex.height >> mip_level,
            depth: ex.depth >> mip_level,
        }
    }
    /// Returns true if the given values are within the image's array layers.
    pub fn array_bounds_check(
        &self, base_array_layer: u32, layer_count: u32,
    ) -> bool {
        self.array_layers >= base_array_layer
            && self.array_layers - base_array_layer >= layer_count
    }
    /// Returns true if the given point is within the image at the given mip
    /// level.
    pub fn offset_bounds_check(
        &self, mip_level: u32, offset: Offset3D,
    ) -> bool {
        let ex = self.extent(mip_level);
        mip_level < self.mip_levels
            && (offset.x >= 0 && offset.y >= 0 && offset.z >= 0)
            && ex.width >= offset.x as u32
            && ex.height >= offset.y as u32
            && ex.depth >= offset.z as u32
    }
    /// Returns true if the given rectangle is within the image at the given mip
    /// level.
    pub fn bounds_check(
        &self, mip_level: u32, offset: Offset3D, extent: Extent3D,
    ) -> bool {
        let ex = self.extent(mip_level);
        self.offset_bounds_check(mip_level, offset)
            && ex.width - offset.x as u32 >= extent.width
            && ex.height - offset.y as u32 >= extent.height
            && ex.depth - offset.z as u32 >= extent.depth
    }
}

impl Drop for ImageWithoutMemory<'_> {
    fn drop(&mut self) {
        if self.owned {
            unsafe {
                (self.device.fun.destroy_image)(
                    self.device.handle(),
                    self.handle.borrow_mut(),
                    None,
                )
            }
        }
    }
}

impl<'a> Image<'a> {
    /// Note that it is an error to bind a storage image to host-visible memory
    /// when robust buffer access is not enabled.
    #[doc = crate::man_link!(vkBindImageMemory)]
    pub fn new(
        mut image: ImageWithoutMemory<'a>, memory: &'a DeviceMemory<'a>,
        offset: u64,
    ) -> Result<Self> {
        assert_eq!(memory.device(), image.device);
        if !memory.check(offset, image.memory_requirements()) {
            return Err(Error::InvalidArgument);
        }

        unsafe {
            (memory.device().fun.bind_image_memory)(
                memory.device().handle(),
                image.handle_mut(),
                memory.handle(),
                offset,
            )?;
        }
        Ok(Self(image))
    }

    /// Create an unowned image, for use by the swapchain. The caller must give
    /// the result an appropriate lifetime.
    pub(crate) unsafe fn new_from(
        handle: Handle<VkImage>, device: &'a Device, format: Format,
        extent: Extent3D, array_layers: u32, usage: ImageUsageFlags,
    ) -> Self {
        Self(ImageWithoutMemory {
            handle,
            device,
            format,
            extent,
            array_layers,
            usage,
            mip_levels: 1,
            owned: false,
        })
    }
}

/// An
#[doc = crate::spec_link!("image view", "12", "resources-image-views")]
#[derive(Debug)]
pub struct ImageView<'a> {
    handle: Handle<VkImageView>,
    image: &'a Image<'a>,
}

#[doc = crate::man_link!(VkImageViewCreateInfo)]
#[derive(Default)]
pub struct ImageViewCreateInfo {
    pub flags: ImageViewCreateFlags,
    pub view_type: ImageViewType,
    pub format: Format,
    pub components: ComponentMapping,
    pub subresource_range: ImageSubresourceRange,
}

impl<'a> ImageView<'a> {
    /// Create an image view of the image.
    pub fn new(
        image: &'a Image<'a>, info: &ImageViewCreateInfo,
    ) -> Result<Self> {
        let vk_info = VkImageViewCreateInfo {
            stype: Default::default(),
            next: Default::default(),
            flags: info.flags,
            image: image.handle(),
            view_type: info.view_type,
            format: info.format,
            components: info.components,
            subresource_range: info.subresource_range,
        };
        let mut handle = None;
        unsafe {
            (image.device.fun.create_image_view)(
                image.device.handle(),
                &vk_info,
                None,
                &mut handle,
            )?;
        }
        Ok(Self { handle: handle.unwrap(), image })
    }
}

impl Drop for ImageView<'_> {
    fn drop(&mut self) {
        unsafe {
            (self.image.device().fun.destroy_image_view)(
                self.image.device().handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

impl<'d> ImageView<'d> {
    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkImageView> {
        self.handle.borrow()
    }
    /// Returns the associated device.
    pub fn device(&self) -> &Device {
        self.image.device()
    }
    /// Returns the underlying image
    pub fn image(&'d self) -> &Image {
        &self.image
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::vk;
    #[test]
    fn wrong_mem() {
        let (dev, _) = crate::test_device().unwrap();
        let buf = vk::ImageWithoutMemory::new(
            &dev,
            &ImageCreateInfo {
                extent: Extent3D { width: 64, height: 64, depth: 1 },
                ..Default::default()
            },
        )
        .unwrap();
        assert!(buf.allocate_memory(31).is_err());
    }
    #[test]
    fn require_robust() {
        let inst = vk::Instance::new(&Default::default()).unwrap();
        let (dev, _) = vk::Device::new(
            &inst.enumerate_physical_devices().unwrap()[0],
            &vk::DeviceCreateInfo {
                queue_create_infos: vk::slice(&[vk::DeviceQueueCreateInfo {
                    queue_priorities: vk::slice(&[1.0]),
                    ..Default::default()
                }]),
                ..Default::default()
            },
        )
        .unwrap();
        let buf = vk::ImageWithoutMemory::new(
            &dev,
            &ImageCreateInfo {
                extent: Extent3D { width: 64, height: 64, depth: 1 },
                usage: vk::ImageUsageFlags::STORAGE,
                ..Default::default()
            },
        )
        .unwrap();
        let host_mem = dev
            .physical_device()
            .memory_properties()
            .memory_types
            .iter()
            .position(|ty| {
                ty.property_flags
                    .contains(vk::MemoryPropertyFlags::HOST_VISIBLE)
            })
            .unwrap();
        assert!(buf.allocate_memory(host_mem as u32).is_err());
    }
}
