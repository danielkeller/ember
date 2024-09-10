// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::collections::HashSet;
use std::mem::MaybeUninit;
use std::ops::Range;

use crate::descriptor_set::DescriptorSetLayout;
use crate::device::Device;
use crate::enums::*;
use crate::enums::{PipelineLayoutCreateFlags, ShaderStageFlags};
use crate::error::{Error, Result};
use crate::ffi::*;
use crate::render_pass::RenderPass;
use crate::types::*;

// Maybe there's a better way to manage these layout things.

/// A
#[doc = crate::spec_link!("pipeline layout", "14", "descriptorsets-pipelinelayout")]
#[derive(Debug)]
pub struct PipelineLayout {
    handle: Handle<VkPipelineLayout>,
    // TODO: Use something like RenderPassCompat instead.
    set_layouts: Vec<DescriptorSetLayout>,
    push_constant_ranges: Vec<PushConstantRange>,
    // Wait, why did we need this again?
    push_constant_voids: Vec<Range<u32>>,
    device: Device,
}

impl PipelineLayout {
    #[doc = crate::man_link!(vkCreatePipelineLayout)]
    pub fn new(
        device: &Device, flags: PipelineLayoutCreateFlags,
        set_layouts: &[DescriptorSetLayout],
        push_constant_ranges: Vec<PushConstantRange>,
    ) -> Result<PipelineLayout> {
        let lim = &device.limits();
        if set_layouts.len() > lim.max_bound_descriptor_sets as usize {
            return Err(Error::LimitExceeded);
        }
        for stage in [
            ShaderStageFlags::COMPUTE,
            ShaderStageFlags::VERTEX,
            ShaderStageFlags::FRAGMENT,
        ] {
            let of = |ty| matching_resources(set_layouts, ty, stage);
            let sampler = of(DescriptorType::SAMPLER);
            let image = of(DescriptorType::SAMPLED_IMAGE);
            let image_sampler = of(DescriptorType::COMBINED_IMAGE_SAMPLER);
            let texel_buf = of(DescriptorType::UNIFORM_TEXEL_BUFFER);
            let storage_image = of(DescriptorType::STORAGE_IMAGE);
            let storage_texel_buf = of(DescriptorType::STORAGE_TEXEL_BUFFER);
            let uniform = of(DescriptorType::UNIFORM_BUFFER);
            let uniform_dyn = of(DescriptorType::UNIFORM_BUFFER_DYNAMIC);
            let storage = of(DescriptorType::STORAGE_BUFFER);
            let storage_dyn = of(DescriptorType::STORAGE_BUFFER_DYNAMIC);
            let input = of(DescriptorType::INPUT_ATTACHMENT);
            if sampler + image_sampler > lim.max_per_stage_descriptor_samplers
                || uniform + uniform_dyn
                    > lim.max_per_stage_descriptor_uniform_buffers
                || storage + storage_dyn
                    > lim.max_per_stage_descriptor_storage_buffers
                || image + image_sampler + texel_buf
                    > lim.max_per_stage_descriptor_sampled_images
                || storage_image + storage_texel_buf
                    > lim.max_per_stage_descriptor_storage_images
                || input > lim.max_per_stage_descriptor_input_attachments
                || (sampler + image + image_sampler + texel_buf + storage_image)
                    + (storage_texel_buf + uniform + uniform_dyn + storage)
                    + (storage_dyn + input)
                    > lim.max_per_stage_resources
            {
                return Err(Error::LimitExceeded);
            }
        }
        {
            let of = |ty| {
                matching_resources(&set_layouts, ty, ShaderStageFlags::ALL)
            };
            let sampler = of(DescriptorType::SAMPLER);
            let image = of(DescriptorType::SAMPLED_IMAGE);
            let image_sampler = of(DescriptorType::COMBINED_IMAGE_SAMPLER);
            let texel_buf = of(DescriptorType::UNIFORM_TEXEL_BUFFER);
            let storage_image = of(DescriptorType::STORAGE_IMAGE);
            let storage_texel_buf = of(DescriptorType::STORAGE_TEXEL_BUFFER);
            let uniform = of(DescriptorType::UNIFORM_BUFFER);
            let uniform_dyn = of(DescriptorType::UNIFORM_BUFFER_DYNAMIC);
            let storage = of(DescriptorType::STORAGE_BUFFER);
            let storage_dyn = of(DescriptorType::STORAGE_BUFFER_DYNAMIC);
            let input = of(DescriptorType::INPUT_ATTACHMENT);
            if sampler + image_sampler > lim.max_descriptor_set_samplers
                || uniform + uniform_dyn
                    > lim.max_per_stage_descriptor_uniform_buffers
                || uniform_dyn > lim.max_descriptor_set_uniform_buffers_dynamic
                || storage + storage_dyn
                    > lim.max_descriptor_set_storage_buffers
                || storage_dyn > lim.max_descriptor_set_storage_buffers_dynamic
                || image + image_sampler + texel_buf
                    > lim.max_descriptor_set_sampled_images
                || storage_image + storage_texel_buf
                    > lim.max_descriptor_set_storage_images
                || input > lim.max_descriptor_set_input_attachments
            {
                return Err(Error::LimitExceeded);
            }
        }
        for range in &push_constant_ranges {
            let max = lim.max_push_constants_size;
            if max < range.offset || max - range.offset < range.size {
                return Err(Error::LimitExceeded);
            }
        }
        let mut handle = None;
        unsafe {
            let set_layouts =
                &set_layouts.iter().map(|l| l.borrow()).collect::<Vec<_>>();
            (device.fun().create_pipeline_layout)(
                device.handle(),
                &PipelineLayoutCreateInfo {
                    flags,
                    set_layouts: set_layouts.into(),
                    push_constant_ranges: slice(&push_constant_ranges),
                    ..Default::default()
                },
                None,
                &mut handle,
            )?;
        }
        let set_layouts = set_layouts.iter().map(|s| s.clone()).collect();
        let push_constant_voids = find_voids(&push_constant_ranges)?;
        Ok(PipelineLayout {
            handle: handle.unwrap(),
            set_layouts,
            push_constant_ranges,
            push_constant_voids,
            device: device.clone(),
        })
    }
}

fn find_voids(ranges: &[PushConstantRange]) -> Result<Vec<Range<u32>>> {
    let mut result = vec![0..u32::MAX];
    for range in ranges {
        let end =
            range.offset.checked_add(range.size).ok_or(Error::OutOfBounds)?;
        let mut result1 = vec![];
        for void in result {
            if range.offset > void.start && end < void.end {
                result1.push(end..void.end);
                result1.push(void.start..range.offset);
            } else if range.offset > void.start {
                result1.push(void.start..range.offset.min(void.end));
            } else {
                result1.push(void.start.max(end)..void.end);
            }
        }
        result = result1;
    }
    Ok(result)
}

fn matching_resources(
    sets: &[DescriptorSetLayout], descriptor_type: DescriptorType,
    stage_flags: ShaderStageFlags,
) -> u32 {
    sets.iter().map(|s| s.num_bindings(descriptor_type, stage_flags)).sum()
}

impl Drop for PipelineLayout {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun().destroy_pipeline_layout)(
                self.device.handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

impl PipelineLayout {
    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkPipelineLayout> {
        self.handle.borrow()
    }
    /// Returns the list of descriptor set layouts.
    pub(crate) fn layouts(&self) -> &[DescriptorSetLayout] {
        &self.set_layouts
    }
    /// Checks that the push constants are in bounds and `stage_flags` are
    /// correct.
    pub(crate) fn bounds_check_push_constants(
        &self, stage_flags: ShaderStageFlags, offset: u32, size: u32,
    ) -> bool {
        let (end, overflow) = offset.overflowing_add(size);
        if overflow {
            return false;
        }
        for void in &self.push_constant_voids {
            if void.start < end && offset < void.end {
                return false;
            }
        }
        for range in &self.push_constant_ranges {
            if range.offset < end
                && offset < range.offset + range.size
                && stage_flags & range.stage_flags != range.stage_flags
            {
                return false;
            }
        }
        true
    }
}

/// A
#[doc = crate::spec_link!("pipeline", "10", "pipelines")]
#[derive(Debug)]
pub struct Pipeline {
    handle: Handle<VkPipeline>,
    layout: Arc<PipelineLayout>,
    render_pass: Option<Arc<RenderPass>>,
    subpass: u32,
}

#[doc = crate::man_link!(VkGraphicsPipelineCreateInfo)]
pub struct GraphicsPipelineCreateInfo<'a> {
    pub stages: &'a [PipelineShaderStageCreateInfo<'a>],
    pub vertex_input_state: &'a PipelineVertexInputStateCreateInfo<'a>,
    pub input_assembly_state: &'a PipelineInputAssemblyStateCreateInfo,
    pub tessellation_state: Option<&'a PipelineTessellationStateCreateInfo>,
    pub viewport_state: &'a PipelineViewportStateCreateInfo<'a>,
    pub rasterization_state: &'a PipelineRasterizationStateCreateInfo,
    pub multisample_state: &'a PipelineMultisampleStateCreateInfo<'a>,
    pub depth_stencil_state: Option<&'a PipelineDepthStencilStateCreateInfo>,
    pub color_blend_state: &'a PipelineColorBlendStateCreateInfo<'a>,
    pub dynamic_state: Option<&'a PipelineDynamicStateCreateInfo<'a>>,
    pub layout: &'a Arc<PipelineLayout>,
    pub render_pass: &'a Arc<RenderPass>,
    pub subpass: u32,
}

impl Pipeline {
    // TODO: Bulk create
    /// Returns [`Error::OutOfBounds`] if `info.subpass` is out of bounds of
    /// `info.render_pass`, or the specialization constants are out of bounds.
    /// Returns [`Error::InvalidArgument`] if any vertex input binding number are
    /// repeated, any vertex attribute locations are repeated, or any vertex
    /// attributes refer to a nonexistent binding.
    #[doc = crate::man_link!(vkCreateGraphicsPipeline)]
    pub fn new_graphics(
        info: &GraphicsPipelineCreateInfo<'_>,
    ) -> Result<Arc<Self>> {
        let lim = info.render_pass.device.limits();
        if info.subpass >= info.render_pass.num_subpasses() {
            return Err(Error::OutOfBounds);
        }
        let mut bindings = HashSet::new();
        for b in info.vertex_input_state.vertex_binding_descriptions {
            if b.binding > lim.max_vertex_input_bindings
                || b.stride > lim.max_vertex_input_binding_stride
            {
                return Err(Error::LimitExceeded);
            }
            if !bindings.insert(b.binding) {
                return Err(Error::InvalidArgument);
            }
        }
        let mut locations = HashSet::new();
        for att in info.vertex_input_state.vertex_attribute_descriptions {
            if att.location > lim.max_vertex_input_attributes
                || att.offset > lim.max_vertex_input_attribute_offset
            {
                return Err(Error::LimitExceeded);
            }
            if !locations.insert(att.location)
                || !bindings.contains(&att.binding)
            {
                return Err(Error::InvalidArgument);
            }
        }
        if info.viewport_state.viewports.len() > lim.max_viewports {
            return Err(Error::LimitExceeded);
        }
        for viewport in info.viewport_state.viewports {
            if viewport.height as u32 > lim.max_viewport_dimensions[0]
                || viewport.width as u32 > lim.max_viewport_dimensions[1]
                || viewport.x < lim.viewport_bounds_range[0]
                || viewport.y < lim.viewport_bounds_range[0]
                || viewport.x + viewport.width > lim.viewport_bounds_range[1]
                || viewport.y + viewport.height > lim.viewport_bounds_range[1]
            {
                return Err(Error::LimitExceeded);
            }
        }
        for stage in info.stages {
            check_specialization_constants(stage)?;
        }
        let vk_info = VkGraphicsPipelineCreateInfo {
            stype: Default::default(),
            next: Default::default(),
            flags: Default::default(),
            stages: info.stages.into(),
            vertex_input_state: info.vertex_input_state,
            input_assembly_state: info.input_assembly_state,
            tessellation_state: info.tessellation_state,
            viewport_state: info.viewport_state,
            rasterization_state: info.rasterization_state,
            multisample_state: info.multisample_state,
            depth_stencil_state: info.depth_stencil_state,
            color_blend_state: info.color_blend_state,
            dynamic_state: info.dynamic_state,
            layout: info.layout.handle(),
            render_pass: info.render_pass.handle(),
            subpass: info.subpass,
            base_pipeline_handle: Default::default(),
            base_pipeline_index: Default::default(),
        };
        let mut handle = MaybeUninit::uninit();
        unsafe {
            (info.layout.device.fun().create_graphics_pipelines)(
                info.layout.device.handle(),
                None,
                1,
                std::array::from_ref(&vk_info).into(),
                None,
                std::array::from_mut(&mut handle).into(),
            )?;
        }
        Ok(Arc::new(Pipeline {
            handle: unsafe { handle.assume_init() },
            layout: info.layout.clone(),
            render_pass: Some(info.render_pass.clone()),
            subpass: info.subpass,
        }))
    }
    /// Returns [`Error::OutOfBounds`] if the specialization constants are out of
    /// bounds.
    #[doc = crate::man_link!(vkCreateComputePipeline)]
    pub fn new_compute(
        stage: PipelineShaderStageCreateInfo, layout: &Arc<PipelineLayout>,
    ) -> Result<Arc<Pipeline>> {
        check_specialization_constants(&stage)?;
        let info = ComputePipelineCreateInfo {
            stype: Default::default(),
            next: Default::default(),
            flags: Default::default(),
            stage,
            layout: layout.handle(),
            base_pipeline_handle: Default::default(),
            base_pipeline_index: Default::default(),
        };
        let mut handle = MaybeUninit::uninit();
        unsafe {
            (layout.device.fun().create_compute_pipelines)(
                layout.device.handle(),
                None,
                1,
                std::array::from_ref(&info).into(),
                None,
                std::array::from_mut(&mut handle).into(),
            )?;
        }
        Ok(Arc::new(Pipeline {
            handle: unsafe { handle.assume_init() },
            layout: layout.clone(),
            render_pass: None,
            subpass: 0,
        }))
    }
}

impl Pipeline {
    /// Borrows the inner Vulkan handle.
    pub fn borrow(&self) -> Ref<VkPipeline> {
        self.handle.borrow()
    }
    /// Returns the pipeline layout.
    pub(crate) fn layout(&self) -> &PipelineLayout {
        &*self.layout
    }
    /// Returns the render pass the pipeline was created with, if it is a
    /// graphics pipeline.
    pub fn render_pass(&self) -> Option<&RenderPass> {
        self.render_pass.as_deref()
    }
    /// Returns true if the pipeline is compatible with the given render pass
    /// and subpass.
    pub fn is_compatible_with(&self, pass: &RenderPass, subpass: u32) -> bool {
        self.render_pass.as_ref().map_or(false, |p| p.compatible(pass))
            && self.subpass == subpass
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        unsafe {
            (self.layout.device.fun().destroy_pipeline)(
                self.layout.device.handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

fn check_specialization_constants<T>(
    info: &PipelineShaderStageCreateInfo<T>,
) -> Result<()> {
    if let Some(spec) = &info.specialization_info {
        for entry in spec.map_entries {
            if spec.data.len() < entry.offset as usize
                || spec.data.len() - (entry.offset as usize) < entry.size
            {
                return Err(Error::OutOfBounds);
            }
        }
    }
    Ok(())
}

#[cfg(pipeline_cache)]

/// A
#[doc = crate::spec_link!("pipeline cache", "10", "pipelines-cache")]
pub struct PipelineCache<'d> {
    handle: Handle<VkPipelineCache>,
    device: &'d Device,
}

#[cfg(pipeline_cache)]
impl<'d> PipelineCache<'d> {
    /// Safety: `data` must either be empty or have been retuned from a previous
    /// call to [`PipelineCache::data`]. Hilariously, this function is
    /// actually impossible to make safe; Vulkan provides no way to validate the
    /// cache data, and the data is generally written to a file where it could
    /// be damaged or altered. Caveat emptor.
    ///
    #[doc = crate::man_link!(vkCreatePipelineCache)]
    pub unsafe fn new(device: &'d Device, data: &[u8]) -> Result<Self> {
        let mut handle = None;
        let info = PipelineCacheCreateInfo {
            stype: Default::default(),
            next: Default::default(),
            flags: Default::default(),
            initial_data: data.into(),
        };
        (device.fun().create_pipeline_cache)(
            device.handle(),
            &info,
            None,
            &mut handle,
        )?;
        Ok(Self { handle: handle.unwrap(), device })
    }

    /// Returns the data in the pipeline cache.
    pub fn data(&self) -> Result<Vec<u8>> {
        let mut len = 0;
        let mut result = Vec::new();
        loop {
            unsafe {
                (self.device.fun().get_pipeline_cache_data)(
                    self.device.handle(),
                    self.handle.borrow(),
                    &mut len,
                    None,
                )?;
                result.reserve(len);
                let maybe_worked = (self.device.fun().get_pipeline_cache_data)(
                    self.device.handle(),
                    self.handle.borrow(),
                    &mut len,
                    ArrayMut::from_slice(result.spare_capacity_mut()),
                );
                if let Err(err) = maybe_worked {
                    if let Error::Incomplete = err.into() {
                        continue; // Racing pipeline creation
                    }
                }
                maybe_worked?;
                break;
            }
        }
        unsafe { result.set_len(len) };
        Ok(result)
    }
}

#[cfg(pipeline_cache)]
impl Drop for PipelineCache<'_> {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun().destroy_pipeline_cache)(
                self.device.handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}
