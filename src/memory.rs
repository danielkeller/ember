// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::marker::PhantomData;
use std::ptr::NonNull;

use crate::error::{Error, Result};
use crate::types::*;
use crate::vk::Device;

/// A piece of
#[doc = crate::spec_link!("device memory", "11", "memory-device")]
#[derive(Debug)]
pub struct DeviceMemory<'d> {
    handle: Handle<VkDeviceMemory>,
    device: &'d Device<'d>,
    allocation_size: u64,
    memory_type_index: u32,
}

impl<'d> DeviceMemory<'d> {
    /// Returns [`Error::OutOfBounds`] if no memory type exists with the given
    /// index.
    #[doc = crate::man_link!(vkAllocateMemory)]
    pub fn new(
        device: &'d Device, allocation_size: u64, memory_type_index: u32,
    ) -> Result<Self> {
        let mem_types = device.physical_device().memory_properties();
        if memory_type_index >= mem_types.memory_types.len() {
            return Err(Error::OutOfBounds);
        }
        device.increment_memory_alloc_count()?;
        let mut handle = None;
        let result = unsafe {
            (device.fun.allocate_memory)(
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
        Ok(Self {
            handle: handle.unwrap(),
            device,
            allocation_size,
            memory_type_index,
        })
    }

    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkDeviceMemory> {
        self.handle.borrow()
    }
    /// Borrows the inner Vulkan handle.
    pub fn mut_handle(&mut self) -> Mut<VkDeviceMemory> {
        self.handle.borrow_mut()
    }
    /// Returns the associated device.
    pub fn device(&self) -> &Device {
        self.device
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
}

impl Drop for DeviceMemory<'_> {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun.free_memory)(
                self.device.handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
        self.device.decrement_memory_alloc_count();
    }
}

/// A [`DeviceMemory`] which has been mapped and can be written to
pub struct MappedMemory<'d> {
    memory: DeviceMemory<'d>,
    size: usize,
    ptr: NonNull<u8>,
}

/// A structure for copying data out of mapped memory. Implements
/// [`std::io::Read`].
pub struct MemoryRead<'a> {
    ptr: NonNull<u8>,
    end: *const u8,
    _lt: PhantomData<&'a ()>,
}
/// A structure for copying data into mapped memory. Implements
/// [`std::io::Write`].
pub struct MemoryWrite<'a> {
    ptr: NonNull<u8>,
    end: *const u8,
    _lt: PhantomData<&'a ()>,
}

#[allow(clippy::len_without_is_empty)]
impl<'d> DeviceMemory<'d> {
    /// Map the memory so it can be written to. Returns [`Error::OutOfBounds`] if
    /// `offset` and `size` are out of bounds. Currently, memory cannot be mapped
    /// or unmapped while buffers are bound to it.
    pub fn map(mut self, offset: u64, size: usize) -> Result<MappedMemory<'d>> {
        let (end, overflow) = offset.overflowing_add(size as u64);
        if overflow || end > self.allocation_size || size > isize::MAX as usize
        {
            return Err(Error::OutOfBounds);
        }
        let mut ptr = std::ptr::null_mut();
        unsafe {
            (self.device.fun.map_memory)(
                self.device.handle(),
                self.handle.borrow_mut(),
                offset,
                size as u64,
                Default::default(),
                &mut ptr,
            )?;
        }
        Ok(MappedMemory { memory: self, size, ptr: NonNull::new(ptr).unwrap() })
    }
}

impl<'d> std::ops::Deref for MappedMemory<'d> {
    type Target = DeviceMemory<'d>;

    fn deref(&self) -> &Self::Target {
        &self.memory
    }
}

impl<'d> MappedMemory<'d> {
    /// Unmaps the memory.
    pub fn unmap(mut self) -> DeviceMemory<'d> {
        unsafe {
            (self.device.fun.unmap_memory)(
                self.device.handle(),
                self.memory.handle.borrow_mut(),
            )
        }
        self.memory
    }

    /// Read the memory's contents. It may be garbage (although it won't be
    /// uninitialized). If `offset` is out of bounds, the result will be empty.
    #[inline]
    pub fn read_at(&self, offset: usize) -> MemoryRead {
        unsafe {
            let ptr = self.ptr.as_ptr().add(offset.min(self.size));
            MemoryRead {
                ptr: NonNull::new_unchecked(ptr),
                end: self.ptr.as_ptr().add(self.size),
                _lt: PhantomData,
            }
        }
    }

    /// Write to the memory. If `offset` is out of bounds, the result will be
    /// empty.
    #[inline]
    pub fn write_at(&mut self, offset: usize) -> MemoryWrite {
        unsafe {
            let ptr = self.ptr.as_ptr().add(offset.min(self.size));
            MemoryWrite {
                ptr: NonNull::new_unchecked(ptr),
                end: self.ptr.as_ptr().add(self.size),
                _lt: PhantomData,
            }
        }
    }
}

impl<'a> std::io::Read for MemoryRead<'a> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        unsafe {
            let size = self.end.offset_from(self.ptr.as_ptr()) as usize;
            let count = size.min(buf.len());
            std::ptr::copy_nonoverlapping(
                self.ptr.as_ptr(),
                buf.as_mut_ptr(),
                count,
            );
            self.ptr = NonNull::new_unchecked(self.ptr.as_ptr().add(count));
            Ok(count)
        }
    }
    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        unsafe {
            let size = self.end.offset_from(self.ptr.as_ptr()) as usize;
            buf.reserve_exact(size);
            std::ptr::copy_nonoverlapping(
                self.ptr.as_ptr(),
                buf.spare_capacity_mut().as_mut_ptr() as *mut u8,
                size,
            );
            buf.set_len(buf.len() + size);
            Ok(size)
        }
    }
}

impl<'a> std::io::Write for MemoryWrite<'a> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        unsafe {
            let size = self.end.offset_from(self.ptr.as_ptr()) as usize;
            let count = size.min(buf.len());
            std::ptr::copy_nonoverlapping(
                buf.as_ptr(),
                self.ptr.as_ptr(),
                count,
            );
            self.ptr = NonNull::new_unchecked(self.ptr.as_ptr().add(count));
            Ok(count)
        }
    }
    /// Returns an error of kind [`std::io::ErrorKind::WriteZero`] if not all
    /// the bytes could be written.
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        if self.write(buf)? == buf.len() {
            Ok(())
        } else {
            Err(std::io::ErrorKind::WriteZero.into())
        }
    }

    /// Does nothing.
    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

// Access to ptr is properly controlled with borrows
unsafe impl Send for MappedMemory<'_> {}
unsafe impl Sync for MappedMemory<'_> {}
impl std::panic::UnwindSafe for MappedMemory<'_> {}
impl std::panic::RefUnwindSafe for MappedMemory<'_> {}
unsafe impl<'a> Send for MemoryRead<'a> {}
unsafe impl<'a> Sync for MemoryRead<'a> {}
impl<'a> std::panic::UnwindSafe for MemoryRead<'a> {}
impl<'a> std::panic::RefUnwindSafe for MemoryRead<'a> {}
unsafe impl<'a> Send for MemoryWrite<'a> {}
unsafe impl<'a> Sync for MemoryWrite<'a> {}
impl<'a> std::panic::UnwindSafe for MemoryWrite<'a> {}
impl<'a> std::panic::RefUnwindSafe for MemoryWrite<'a> {}
