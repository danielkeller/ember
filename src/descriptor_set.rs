// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt::Debug;
use std::mem::MaybeUninit;

use crate::device::Device;
use crate::enums::{DescriptorType, ShaderStageFlags};
use crate::error::{Error, Result};
use crate::ffi::Array;
use crate::types::*;

pub mod update;

/// A
#[doc = crate::spec_link!("descriptor set layout", "14", "descriptorsets-setlayout")]
#[derive(Debug, Eq)]
pub struct DescriptorSetLayout<'d> {
    handle: Handle<VkDescriptorSetLayout>,
    bindings: Vec<DescriptorSetLayoutBinding<'d>>,
    device: &'d Device<'d>,
}

/// Note that unlike in Vulkan, the binding number is implicitly the index of
/// the array that is passed into [`DescriptorSetLayout::new`].
/// If non-consecutive binding numbers are desired, create dummy descriptors to
/// fill the gaps.
///
/// For [`DescriptorType::COMBINED_IMAGE_SAMPLER`], currently the use of
/// immutable samplers is required.
///
#[doc = crate::man_link!(VkDescriptorSetLayoutBinding)]
#[derive(Debug, PartialEq, Eq, Default)]
pub struct DescriptorSetLayoutBinding<'a> {
    pub descriptor_type: DescriptorType,
    pub descriptor_count: u32,
    pub stage_flags: ShaderStageFlags,
    pub immutable_samplers: Vec<Ref<'a, VkSampler>>,
}

impl<'d> DescriptorSetLayout<'d> {
    // TODO: vkGetDescriptorSetLayoutSupport

    #[doc = crate::man_link!(VkDescriptorSetLayout)]
    pub fn new(
        device: &'d Device, bindings: Vec<DescriptorSetLayoutBinding<'d>>,
    ) -> Result<Self> {
        for b in &bindings {
            if !b.immutable_samplers.is_empty()
                && b.immutable_samplers.len() as u32 != b.descriptor_count
            {
                return Err(Error::InvalidArgument);
            }
            if b.descriptor_type == DescriptorType::COMBINED_IMAGE_SAMPLER
                && b.immutable_samplers.is_empty()
            {
                return Err(Error::InvalidArgument);
            }
        }

        let vk_bindings = bindings
            .iter()
            .enumerate()
            .map(|(i, b)| VkDescriptorSetLayoutBinding {
                binding: i as u32,
                descriptor_type: b.descriptor_type,
                descriptor_count: b.descriptor_count,
                stage_flags: b.stage_flags,
                immutable_samplers: Array::from_slice(&b.immutable_samplers),
            })
            .collect::<Vec<_>>();
        let mut handle = None;
        unsafe {
            (device.fun.create_descriptor_set_layout)(
                device.handle(),
                &VkDescriptorSetLayoutCreateInfo {
                    bindings: vk_bindings.as_slice().into(),
                    ..Default::default()
                },
                None,
                &mut handle,
            )?;
        }

        Ok(DescriptorSetLayout { handle: handle.unwrap(), bindings, device })
    }
}

impl Drop for DescriptorSetLayout<'_> {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun.destroy_descriptor_set_layout)(
                self.device.handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

impl PartialEq for DescriptorSetLayout<'_> {
    /// Compatible descriptor sets layouts are equal
    fn eq(&self, other: &Self) -> bool {
        self.bindings == other.bindings && self.device == other.device
    }
}

impl DescriptorSetLayout<'_> {
    /// Borrows the inner Vulkan handle.
    pub fn borrow(&self) -> Ref<VkDescriptorSetLayout> {
        self.handle.borrow()
    }
    /// Returns the number of dynamic offsets the descriptor set will require.
    pub(crate) fn num_dynamic_offsets(&self) -> u32 {
        let mut result = 0;
        for b in &self.bindings {
            if b.descriptor_type == DescriptorType::UNIFORM_BUFFER_DYNAMIC
                || b.descriptor_type == DescriptorType::STORAGE_BUFFER_DYNAMIC
            {
                result += b.descriptor_count
            }
        }
        result
    }
    /// Returns the number of bindings of the specified type and stage.
    pub(crate) fn num_bindings(
        &self, descriptor_type: DescriptorType, stage_flags: ShaderStageFlags,
    ) -> u32 {
        self.bindings
            .iter()
            .filter(|b| {
                b.descriptor_type == descriptor_type
                    && !(b.stage_flags & stage_flags).is_empty()
            })
            .count() as u32
    }
}

/// A
#[doc = crate::spec_link!("descriptor pool", "14", "descriptorsets-allocation")]
pub struct DescriptorPool<'d> {
    handle: Handle<VkDescriptorPool>,
    device: &'d Device<'d>,
    scratch: bumpalo::Bump,
}

impl<'d> DescriptorPool<'d> {
    #[doc = crate::man_link!(vkCreateDescriptorPool)]
    pub fn new(
        device: &'d Device, max_sets: u32, pool_sizes: &[DescriptorPoolSize],
    ) -> Result<Self> {
        let mut handle = None;
        unsafe {
            (device.fun.create_descriptor_pool)(
                device.handle(),
                &DescriptorPoolCreateInfo {
                    max_sets,
                    pool_sizes: pool_sizes.into(),
                    ..Default::default()
                },
                None,
                &mut handle,
            )?;
        }
        Ok(DescriptorPool {
            handle: handle.unwrap(),
            device,
            scratch: bumpalo::Bump::new(),
        })
    }
}

impl Drop for DescriptorPool<'_> {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun.destroy_descriptor_pool)(
                self.device.handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

impl DescriptorPool<'_> {
    pub fn reset(&mut self) -> Result<()> {
        unsafe {
            (self.device.fun.reset_descriptor_pool)(
                self.device.handle(),
                self.handle.borrow_mut(),
                Default::default(),
            )?;
        }
        Ok(())
    }
}

// Theoretically the descriptor set layout can be destroyed while the set is
// still in use, only creating and updating the set need it. But the immutable
// samplers still need to outlive the set, and this shorter lifetime is tricky
// to allow, so we force the layout to outlive the set as well.

/// A
#[doc = concat!(crate::spec_link!("descriptor set", "14", "descriptorsets-sets"), ".")]
///
/// The descriptor set may be written to by creating a
/// [`DescriptorSetUpdateBuilder`](crate::vk::DescriptorSetUpdateBuilder).
///
/// Any resources that are written into the descriptor set have their reference
/// count incremented and held by the set. To decrement the count and allow the
/// resources to be freed, either the descriptor must be overwritten with
/// another resource, or the descriptor set must be dropped. (Note that calling
/// [`bind_descriptor_sets`](crate::command_buffer::CommandRecording::bind_descriptor_sets)
/// will prevent the set from being freed until the command pool is
/// [`reset`](crate::command_buffer::CommandPool::reset).)
#[derive(Debug)]
pub struct DescriptorSet<'a> {
    handle: Handle<VkDescriptorSet>,
    layout: &'a DescriptorSetLayout<'a>,
    inited: &'a mut [&'a mut [bool]],
}

impl<'a> DescriptorSet<'a> {
    #[doc = crate::man_link!(vkAllocateDescriptorSets)]
    pub fn new(
        pool: &'a DescriptorPool<'a>, layout: &'a DescriptorSetLayout<'a>,
    ) -> Result<Self> {
        assert_eq!(pool.device, layout.device);
        let mut handle = MaybeUninit::uninit();
        let handle = unsafe {
            (pool.device.fun.allocate_descriptor_sets)(
                pool.device.handle(),
                &DescriptorSetAllocateInfo {
                    stype: Default::default(),
                    next: Default::default(),
                    descriptor_pool: pool.handle.borrow_mut_unchecked(),
                    set_layouts: (&[layout.borrow()]).into(),
                },
                std::array::from_mut(&mut handle).into(),
            )?;
            handle.assume_init()
        };
        let inited = pool.scratch.alloc_slice_fill_iter(
            layout.bindings.iter().map(|b| {
                pool.scratch
                    .alloc_slice_fill_default(b.descriptor_count as usize)
            }),
        );
        Ok(DescriptorSet { handle, layout, inited })
    }

    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkDescriptorSet> {
        self.handle.borrow()
    }
    /// Mutably borrows the inner Vulkan handle.
    pub fn handle_mut(&mut self) -> Mut<VkDescriptorSet> {
        self.handle.borrow_mut()
    }
    /// Returns the set's layout.
    pub fn layout(&self) -> &DescriptorSetLayout {
        &self.layout
    }
    /// Returns true if every member of the set has had a value written to it.
    pub fn is_initialized(&self) -> bool {
        self.inited.iter().all(|rs| rs.iter().all(|r| *r))
    }
}
