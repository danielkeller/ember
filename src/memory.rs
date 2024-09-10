// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::error::{Error, Result};
use crate::types::*;
use crate::vk::Device;

#[derive(Debug)]
pub(crate) struct MemoryInner {
    handle: Handle<VkDeviceMemory>,
    device: Device,
}

/// A piece of
#[doc = crate::spec_link!("device memory", "11", "memory-device")]
#[derive(Debug)]
pub struct DeviceMemory {
    inner: Arc<MemoryInner>,
    handle: Mut<'static, VkDeviceMemory>,
    allocation_size: u64,
    memory_type_index: u32,
}

#[allow(clippy::len_without_is_empty)]
impl DeviceMemory {
    /// Returns [`Error::OutOfBounds`] if no memory type exists with the given
    /// index.
    #[doc = crate::man_link!(vkAllocateMemory)]
    pub fn new(
        device: &Device, allocation_size: u64, memory_type_index: u32,
    ) -> Result<Self> {
        let mem_types = device.physical_device().memory_properties();
        if memory_type_index >= mem_types.memory_types.len() {
            return Err(Error::OutOfBounds);
        }
        device.increment_memory_alloc_count()?;
        let mut handle = None;
        let result = unsafe {
            (device.fun().allocate_memory)(
                device.handle(),
                &MemoryAllocateInfo {
                    stype: Default::default(),
                    next: Default::default(),
                    allocation_size,
                    memory_type_index,
                },
                None,
                &mut handle,
            )
        };
        if result.is_err() {
            device.decrement_memory_alloc_count();
            result?;
        }
        let inner = Arc::new(MemoryInner {
            handle: handle.unwrap(),
            device: device.clone(),
        });
        let handle = unsafe {
            inner.handle.borrow_mut_unchecked().reborrow_mut_unchecked()
        };
        Ok(Self { inner, handle, allocation_size, memory_type_index })
    }

    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkDeviceMemory> {
        self.handle.reborrow()
    }
    /// Borrows the inner Vulkan handle.
    pub fn mut_handle(&mut self) -> Mut<VkDeviceMemory> {
        self.handle.reborrow_mut()
    }
    /// Returns the associated device.
    pub fn device(&self) -> &Device {
        &self.inner.device
    }
    /// Returns the size of the memory in bytes.
    pub fn len(&self) -> u64 {
        self.allocation_size
    }
    /// Check if the memory meets `requirements` at the given offset.
    pub fn check(&self, offset: u64, requirements: MemoryRequirements) -> bool {
        let (end, overflow) = offset.overflowing_add(requirements.size);
        (1 << self.memory_type_index) & requirements.memory_type_bits != 0
            && offset & (requirements.alignment - 1) == 0
            && !overflow
            && end <= self.allocation_size
    }

    pub(crate) fn inner(&self) -> Arc<MemoryInner> {
        self.inner.clone()
    }
}

impl Drop for MemoryInner {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun().free_memory)(
                self.device.handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
        self.device.decrement_memory_alloc_count();
    }
}

/// A [`DeviceMemory`] which has been mapped and can be written to
pub struct MappedMemory {
    memory: DeviceMemory,
    size: usize,
    ptr: *mut u8,
}

impl DeviceMemory {
    /// Map the memory so it can be written to. Returns [`Error::OutOfBounds`] if
    /// `offset` and `size` are out of bounds.
    pub fn map(mut self, offset: u64, size: usize) -> Result<MappedMemory> {
        let (end, overflow) = offset.overflowing_add(size as u64);
        if overflow || end > self.allocation_size || size > isize::MAX as usize
        {
            return Err(Error::OutOfBounds);
        }
        let mut ptr = std::ptr::null_mut();
        unsafe {
            (self.inner.device.fun().map_memory)(
                self.inner.device.handle(),
                self.handle.reborrow_mut(),
                offset,
                size as u64,
                Default::default(),
                &mut ptr,
            )?;
        }
        Ok(MappedMemory { memory: self, size, ptr })
    }
}

impl std::ops::Deref for MappedMemory {
    type Target = DeviceMemory;
    fn deref(&self) -> &Self::Target {
        &self.memory
    }
}

impl std::ops::DerefMut for MappedMemory {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.memory
    }
}

impl MappedMemory {
    pub fn unmap(mut self) -> DeviceMemory {
        unsafe {
            (self.inner.device.fun().unmap_memory)(
                self.memory.inner.device.handle(),
                self.memory.handle.reborrow_mut(),
            )
        }
        self.memory
    }

    /// Read the memory's contents. It may be garbage (although it won't be
    /// uninitialized).
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.size) }
    }

    /// Write to the memory.
    #[inline]
    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.size) }
    }
}

// Access to ptr is properly controlled with borrows
unsafe impl Send for MappedMemory {}
unsafe impl Sync for MappedMemory {}
impl std::panic::UnwindSafe for MappedMemory {}
impl std::panic::RefUnwindSafe for MappedMemory {}
