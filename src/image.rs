// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::enums::*;
use crate::error::{Error, Result};
use crate::memory::{DeviceMemory, MemoryInner};
use crate::render_pass::{RenderPass, RenderPassCompat};
use crate::types::*;
use crate::vk::Device;

use std::fmt::Debug;

/// An image with no memory. Call [`Image::new`] to bind memory and create an
/// [`Image`].
#[derive(Debug)]
pub struct ImageWithoutMemory {
    handle: Handle<VkImage>,
    format: Format,
    extent: Extent3D,
    mip_levels: u32,
    array_layers: u32,
    usage: ImageUsageFlags,
    owned: bool,
    device: Device,
}

#[derive(Debug)]
struct ImageInner {
    inner: ImageWithoutMemory,
    memory: Option<Arc<MemoryInner>>,
}

/// An
#[doc = crate::spec_link!("image", "12", "resources-images")]
/// with memory attached to it.
#[derive(Debug)]
pub struct Image {
    inner: Arc<ImageInner>,
}

#[derive(Debug)]
struct ImageViewInner {
    handle: Handle<VkImageView>,
    image: Image,
}

/// An
#[doc = crate::spec_link!("image view", "12", "resources-image-views")]
#[derive(Debug)]
pub struct ImageView {
    inner: Arc<ImageViewInner>,
}

/// A
#[doc = crate::spec_link!("framebuffer", "8", "_framebuffers")]
#[derive(Debug)]
pub struct Framebuffer {
    handle: Handle<VkFramebuffer>,
    render_pass_compat: RenderPassCompat,
    attachments: Vec<Arc<ImageViewInner>>,
    device: Device,
}

impl std::ops::Deref for Image {
    type Target = ImageWithoutMemory;

    fn deref(&self) -> &Self::Target {
        &self.inner.inner
    }
}

impl std::ops::Deref for ImageView {
    type Target = Image;

    fn deref(&self) -> &Self::Target {
        &self.inner.image
    }
}

impl ImageWithoutMemory {
    #[doc = crate::man_link!(vkCreateImage)]
    pub fn new(device: &Device, info: &ImageCreateInfo<'_>) -> Result<Self> {
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
            (device.fun().create_image)(
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
            device: device.clone(),
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
        &self.device
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
            (self.device.fun().get_image_memory_requirements)(
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

impl Drop for ImageWithoutMemory {
    fn drop(&mut self) {
        if self.owned {
            unsafe {
                (self.device.fun().destroy_image)(
                    self.device.handle(),
                    self.handle.borrow_mut(),
                    None,
                )
            }
        }
    }
}

impl Image {
    /// Note that it is an error to bind a storage image to host-visible memory
    /// when robust buffer access is not enabled.
    #[doc = crate::man_link!(vkBindImageMemory)]
    pub fn new(
        mut image: ImageWithoutMemory, memory: &DeviceMemory, offset: u64,
    ) -> Result<Self> {
        assert_eq!(memory.device(), &image.device);
        if !memory.check(offset, image.memory_requirements()) {
            return Err(Error::InvalidArgument);
        }

        unsafe {
            (image.device.fun().bind_image_memory)(
                image.device.handle(),
                image.handle.borrow_mut(),
                memory.handle(),
                offset,
            )?;
        }
        let inner =
            Arc::new(ImageInner { inner: image, memory: Some(memory.inner()) });
        Ok(Self { inner })
    }

    /// Create an unowned image, for use by the swapchain. The caller must give
    /// the result an appropriate lifetime.
    pub(crate) unsafe fn new_from(
        handle: Handle<VkImage>, device: Device, format: Format,
        extent: Extent3D, array_layers: u32, usage: ImageUsageFlags,
    ) -> Self {
        let inner = ImageWithoutMemory {
            handle,
            device,
            format,
            extent,
            array_layers,
            usage,
            mip_levels: 1,
            owned: false,
        };
        let inner = Arc::new(ImageInner { inner, memory: None });
        Self { inner }
    }

    // Not publicly cloneable for consistency.
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
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

impl ImageView {
    /// Create an image view of the image.
    pub fn new(image: &Image, info: &ImageViewCreateInfo) -> Result<Self> {
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
            (image.device.fun().create_image_view)(
                image.device.handle(),
                &vk_info,
                None,
                &mut handle,
            )?;
        }
        let inner = Arc::new(ImageViewInner {
            handle: handle.unwrap(),
            image: image.clone(),
        });
        Ok(Self { inner })
    }
}

impl Drop for ImageViewInner {
    fn drop(&mut self) {
        unsafe {
            (self.image.device().fun().destroy_image_view)(
                self.image.device().handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

impl ImageView {
    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkImageView> {
        self.inner.handle.borrow()
    }
}

impl Framebuffer {
    #[doc = crate::man_link!(vkCreateFrameuffer)]
    pub fn new(
        render_pass: &RenderPass, flags: FramebufferCreateFlags,
        attachments: &[&ImageView], size: Extent3D,
    ) -> Result<Self> {
        for iv in attachments {
            assert_eq!(iv.device(), render_pass.device());
        }
        let lim = render_pass.device().limits();
        if size.width > lim.max_framebuffer_width
            || size.height > lim.max_framebuffer_height
            || size.depth > lim.max_framebuffer_layers
        {
            return Err(Error::LimitExceeded);
        }
        let vk_attachments: Vec<_> =
            attachments.iter().map(|iv| iv.handle()).collect();
        let vk_create_info = VkFramebufferCreateInfo {
            stype: Default::default(),
            next: Default::default(),
            flags,
            render_pass: render_pass.handle(),
            attachments: (&vk_attachments).into(),
            width: size.width,
            height: size.height,
            layers: size.depth,
        };
        let mut handle = None;
        unsafe {
            (render_pass.device().fun().create_framebuffer)(
                render_pass.device().handle(),
                &vk_create_info,
                None,
                &mut handle,
            )?;
        }
        let attachments =
            attachments.iter().map(|iv| iv.inner.clone()).collect();
        Ok(Self {
            handle: handle.unwrap(),
            render_pass_compat: render_pass.compat.clone(),
            attachments,
            device: render_pass.device().clone(),
        })
    }

    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkFramebuffer> {
        self.handle.borrow()
    }
    /// Returns true if this framebuffer is compatible with `pass`
    pub fn is_compatible_with(&self, pass: &RenderPass) -> bool {
        self.render_pass_compat == pass.compat
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun().destroy_framebuffer)(
                self.device.handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

#[cfg(test_disabled)]
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
