// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::enums::*;
use crate::error::{Error, Result};
use crate::memory::{DeviceMemory, MemoryInner};
use crate::types::*;
use crate::vk::Device;

/// A buffer with no memory. Call [`Buffer::new`] to bind memory and create a
/// [`Buffer`].
#[derive(Debug)]
pub struct BufferWithoutMemory {
    handle: Handle<VkBuffer>,
    len: u64,
    usage: BufferUsageFlags,
    device: Device,
}

/// A
#[doc = crate::spec_link!("buffer", "12", "resources-buffers")]
/// with memory attached to it.
#[derive(Debug)]
pub struct Buffer {
    inner: BufferWithoutMemory,
    memory: Arc<MemoryInner>,
}

impl std::ops::Deref for Buffer {
    type Target = BufferWithoutMemory;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[allow(clippy::len_without_is_empty)]
impl BufferWithoutMemory {
    #[doc = crate::man_link!(vkCreateBuffer)]
    pub fn new(device: &Device, info: &BufferCreateInfo<'_>) -> Result<Self> {
        let mut handle = None;
        unsafe {
            (device.fun().create_buffer)(
                device.handle(),
                info,
                None,
                &mut handle,
            )?;
        }
        Ok(BufferWithoutMemory {
            handle: handle.unwrap(),
            len: info.size,
            usage: info.usage,
            device: device.clone(),
        })
    }

    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkBuffer> {
        self.handle.borrow()
    }
    /// Borrows the inner Vulkan handle.
    pub fn handle_mut(&mut self) -> Mut<VkBuffer> {
        self.handle.borrow_mut()
    }
    /// Returns the associated device.
    pub fn device(&self) -> &Device {
        &self.device
    }
    /// Returns the buffer length in bytes.
    pub fn len(&self) -> u64 {
        self.len
    }
    /// Returns true if the offset and length are within the buffer's bounds.
    pub fn bounds_check(&self, offset: u64, len: u64) -> bool {
        self.len() >= offset && self.len() - offset >= len
    }
    /// Returns the allowed buffer usages
    pub fn usage(&self) -> BufferUsageFlags {
        self.usage
    }
    /// If [`BufferCreateInfo::usage`] includes an abritrarily indexable buffer
    /// usage type (uniform, storage, vertex, or index) and the robust buffer
    /// access feature was not enabled at device creation, any host-visible
    /// memory types will be removed from the output. Note that on
    /// some physical devices (eg software rasterizers), *all* memory types are
    /// host-visible.
    ///
    #[doc = crate::man_link!(vkGetBufferMemoryRequirements)]
    pub fn memory_requirements(&self) -> MemoryRequirements {
        let mut result = Default::default();
        unsafe {
            (self.device.fun().get_buffer_memory_requirements)(
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
}

impl Drop for BufferWithoutMemory {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun().destroy_buffer)(
                self.device.handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

impl Buffer {
    // TODO: Bulk bind
    /// Note that it is an error to bind a storage, uniform, vertex, or index
    /// buffer to host-visible memory when robust buffer access is not enabled.
    #[doc = crate::man_link!(vkBindBufferMemory)]
    pub fn new(
        mut buffer: BufferWithoutMemory, memory: &DeviceMemory, offset: u64,
    ) -> Result<Self> {
        assert_eq!(memory.device(), &buffer.device);
        if !memory.check(offset, buffer.memory_requirements()) {
            return Err(Error::InvalidArgument);
        }

        unsafe {
            (memory.device().fun().bind_buffer_memory)(
                memory.device().handle(),
                buffer.handle.borrow_mut(),
                memory.handle(),
                offset,
            )?;
        }
        Ok(Buffer { inner: buffer, memory: memory.inner() })
    }
}

#[cfg(test_disabled)]
mod test {
    use super::*;
    use crate::vk;
    #[test]
    fn wrong_mem() {
        let (dev, _) = crate::test_device().unwrap();
        let buf = vk::BufferWithoutMemory::new(
            &dev,
            &BufferCreateInfo { size: 256, ..Default::default() },
        )
        .unwrap();
        assert!(buf.allocate_memory(31).is_err());
    }
    #[test]
    fn require_robust() {
        let inst = vk::Instance::new(&Default::default()).unwrap();
        let dev = vk::Device::new(
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
        let buf = vk::BufferWithoutMemory::new(
            &dev,
            &BufferCreateInfo {
                size: 256,
                usage: vk::BufferUsageFlags::STORAGE_BUFFER,
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
