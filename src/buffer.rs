// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::enums::*;
use crate::error::{Error, ErrorAndSelf, Result, ResultAndSelf};
use crate::memory::{DeviceMemory, MemoryLifetime};
use crate::subobject::Subobject;
use crate::types::*;
use crate::vk::Device;

/// A buffer with no memory. Call [`Buffer::new`] to bind memory and create a
/// [`Buffer`].
#[derive(Debug)]
pub struct BufferWithoutMemory<'d> {
    handle: Handle<VkBuffer>,
    len: u64,
    usage: BufferUsageFlags,
    device: &'d Device<'d>,
}

/// A
#[doc = crate::spec_link!("buffer", "12", "resources-buffers")]
/// with memory attached to it.
#[derive(Debug)]
pub struct Buffer<'a> {
    inner: BufferWithoutMemory<'a>,
    _memory: Subobject<MemoryLifetime<'a>>,
}

impl<'d> BufferWithoutMemory<'d> {
    #[doc = crate::man_link!(vkCreateBuffer)]
    pub fn new(
        device: &'d Device, info: &BufferCreateInfo<'_>,
    ) -> Result<Self> {
        let mut handle = None;
        unsafe {
            (device.fun.create_buffer)(
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
            device: device,
        })
    }
}
impl<'d> Buffer<'d> {
    // TODO: Bulk bind
    /// Note that it is an error to bind a storage, uniform, vertex, or index
    /// buffer to host-visible memory when robust buffer access is not enabled.
    #[doc = crate::man_link!(vkBindBufferMemory)]
    pub fn new(
        buffer: BufferWithoutMemory<'d>, memory: &DeviceMemory<'d>, offset: u64,
    ) -> ResultAndSelf<Self, BufferWithoutMemory<'d>> {
        assert_eq!(memory.device(), buffer.device);
        if !memory.check(offset, buffer.memory_requirements()) {
            return Err(ErrorAndSelf(Error::InvalidArgument, buffer));
        }
        Self::bind_buffer_impl(buffer, memory, offset)
    }

    fn bind_buffer_impl(
        mut inner: BufferWithoutMemory<'d>, memory: &DeviceMemory<'d>,
        offset: u64,
    ) -> ResultAndSelf<Buffer<'d>, BufferWithoutMemory<'d>> {
        if let Err(err) = unsafe {
            (memory.device().fun.bind_buffer_memory)(
                memory.device().handle(),
                inner.handle.borrow_mut(),
                memory.handle(),
                offset,
            )
        } {
            return Err(ErrorAndSelf(err.into(), inner));
        }
        Ok(Buffer { inner, _memory: memory.resource() })
    }
}

impl Drop for BufferWithoutMemory<'_> {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun.destroy_buffer)(
                self.device.handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

#[allow(clippy::len_without_is_empty)]
impl Buffer<'_> {
    /// Borrows the inner Vulkan handle.
    pub fn borrow(&self) -> Ref<VkBuffer> {
        self.inner.handle.borrow()
    }
    /// Returns the associated device.
    pub fn device(&self) -> &Device {
        self.inner.device
    }
    /// Returns the buffer length in bytes.
    pub fn len(&self) -> u64 {
        self.inner.len
    }
    /// Returns true if the offset and length are within the buffer's bounds.
    pub fn bounds_check(&self, offset: u64, len: u64) -> bool {
        self.len() >= offset && self.len() - offset >= len
    }
    /// Returns the allowed buffer usages
    pub fn usage(&self) -> BufferUsageFlags {
        self.inner.usage
    }
}

impl<'d> BufferWithoutMemory<'d> {
    /// Borrows the inner Vulkan handle.
    pub fn borrow_mut(&mut self) -> Mut<VkBuffer> {
        self.handle.borrow_mut()
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
            (self.device.fun.get_buffer_memory_requirements)(
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
    /// Allocate a single piece of memory for the buffer and bind it. Note that
    /// it is an error to bind a uniform, storage, vertex, or index buffer to
    /// host-visible memory when robust buffer access is not enabled.
    pub fn allocate_memory(
        self, memory_type_index: u32,
    ) -> ResultAndSelf<Buffer<'d>, Self> {
        let mem_req = self.memory_requirements();
        if (1 << memory_type_index) & mem_req.memory_type_bits == 0 {
            return Err(ErrorAndSelf(Error::InvalidArgument, self));
        }
        let memory = match DeviceMemory::new(
            self.device,
            mem_req.size,
            memory_type_index,
        ) {
            Ok(memory) => memory,
            Err(err) => return Err(ErrorAndSelf(err, self)),
        };
        // Don't need to check requirements
        Buffer::bind_buffer_impl(self, &memory, 0)
    }
}

#[cfg(test)]
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
