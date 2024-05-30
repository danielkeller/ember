// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::buffer::Buffer;
use crate::descriptor_set::DescriptorSet;
use crate::enums::*;
use crate::error::{Error, Result};
use crate::ffi::Array;
use crate::pipeline::{Pipeline, PipelineLayout};

use super::{
    Bindings, CommandRecording, RenderPassRecording, SecondaryCommandRecording,
};

impl<'rec, 'pool> RenderPassRecording<'rec, 'pool> {
    /// Binds the pipeline to the appropriate bind point. The reference count of
    /// `pipeline` is incremented.
    #[doc = crate::man_link!(vkCmdBindPipeline)]
    pub fn bind_pipeline(&mut self, pipeline: &'pool Pipeline) {
        self.rec.bind_pipeline(pipeline)
    }
}

impl<'rec, 'pool> SecondaryCommandRecording<'rec, 'pool> {
    /// Binds the pipeline to the appropriate bind point. The reference count of
    /// `pipeline` is incremented.
    #[doc = crate::man_link!(vkCmdBindPipeline)]
    pub fn bind_pipeline(&mut self, pipeline: &'pool Pipeline) {
        self.rec.bind_pipeline(pipeline)
    }
}

impl<'rec, 'pool> CommandRecording<'rec, 'pool> {
    /// Binds the pipeline to the appropriate bind point. The reference count of
    /// `pipeline` is incremented.
    #[doc = crate::man_link!(vkCmdBindPipeline)]
    pub fn bind_pipeline(&mut self, pipeline: &'pool Pipeline) {
        let bind_point;
        if pipeline.render_pass().is_some() {
            self.graphics.pipeline = Some(pipeline);
            bind_point = PipelineBindPoint::GRAPHICS;
        } else {
            self.compute.pipeline = Some(pipeline);
            bind_point = PipelineBindPoint::COMPUTE;
        }
        unsafe {
            (self.device.fun.cmd_bind_pipeline)(
                self.buffer.handle_mut(),
                bind_point,
                pipeline.borrow(),
            )
        }
    }
}

impl<'rec, 'pool> RenderPassRecording<'rec, 'pool> {
    /// Reference counts of buffers are incremented. Returns
    /// [`Error::InvalidArgument`] if `buffers_offsets` is empty or the buffer
    /// usage flags don't include `VERTEX_BUFFER`.
    #[doc = crate::man_link!(vkCmdBindVertexBuffers)]
    pub fn bind_vertex_buffers<const N: usize>(
        &mut self, first_binding: u32, buffers: [&'pool Buffer; N],
        offsets: [u64; N],
    ) -> Result<()> {
        self.rec.bind_vertex_buffers(first_binding, buffers, offsets)
    }
    /// Reference count of `buffer` is incremented. Returns
    /// [`Error::InvalidArgument`] if `buffer` does not have the `INDEX_BUFFER`
    /// usage flag.
    #[doc = crate::man_link!(vkCmdBindIndexBuffer)]
    pub fn bind_index_buffer(
        &mut self, buffer: &'pool Buffer, offset: u64, index_type: IndexType,
    ) -> Result<()> {
        self.rec.bind_index_buffer(buffer, offset, index_type)
    }
}
impl<'rec, 'pool> SecondaryCommandRecording<'rec, 'pool> {
    /// Reference counts of buffers are incremented. Returns
    /// [`Error::InvalidArgument`] if `buffers_offsets` is empty or the buffer
    /// usage flags don't include `VERTEX_BUFFER`.
    #[doc = crate::man_link!(vkCmdBindVertexBuffers)]
    pub fn bind_vertex_buffers<const N: usize>(
        &mut self, first_binding: u32, buffers: [&'pool Buffer; N],
        offsets: [u64; N],
    ) -> Result<()> {
        self.rec.bind_vertex_buffers(first_binding, buffers, offsets)
    }
    /// Reference count of `buffer` is incremented. Returns
    /// [`Error::InvalidArgument`] if `buffer` does not have the `INDEX_BUFFER`
    /// usage flag.
    #[doc = crate::man_link!(vkCmdBindIndexBuffer)]
    pub fn bind_index_buffer(
        &mut self, buffer: &'pool Buffer, offset: u64, index_type: IndexType,
    ) -> Result<()> {
        self.rec.bind_index_buffer(buffer, offset, index_type)
    }
}
impl<'rec, 'pool> CommandRecording<'rec, 'pool> {
    /// Reference counts of buffers are incremented. Returns
    /// [`Error::InvalidArgument`] if `buffers_offsets` is empty or the buffer
    /// usage flags don't include `VERTEX_BUFFER`.
    #[doc = crate::man_link!(vkCmdBindVertexBuffers)]
    pub fn bind_vertex_buffers<const N: usize>(
        &mut self, first_binding: u32, buffers: [&'pool Buffer; N],
        offsets: [u64; N],
    ) -> Result<()> {
        for buffer in buffers {
            if !buffer.usage().contains(BufferUsageFlags::VERTEX_BUFFER) {
                return Err(Error::InvalidArgument);
            }
        }
        let vkbuffers = &buffers.map(|b| b.handle());

        unsafe {
            (self.device.fun.cmd_bind_vertex_buffers)(
                self.buffer.handle_mut(),
                first_binding,
                N as u32,
                vkbuffers.into(),
                (&offsets).into(),
            )
        }
        Ok(())
    }
    /// Reference count of `buffer` is incremented. Returns
    /// [`Error::InvalidArgument`] if `buffer` does not have the `INDEX_BUFFER`
    /// usage flag.
    #[doc = crate::man_link!(vkCmdBindIndexBuffer)]
    pub fn bind_index_buffer(
        &mut self, buffer: &'pool Buffer, offset: u64, index_type: IndexType,
    ) -> Result<()> {
        if !buffer.usage().contains(BufferUsageFlags::INDEX_BUFFER) {
            return Err(Error::InvalidArgument);
        }
        unsafe {
            (self.device.fun.cmd_bind_index_buffer)(
                self.buffer.handle_mut(),
                buffer.handle(),
                offset,
                index_type,
            )
        }
        Ok(())
    }
}

impl<'rec, 'pool> RenderPassRecording<'rec, 'pool> {
    /// Returns [`Error::InvalidArgument`] if a member of `sets` is not compatible
    /// with the corresponding member of `layout`, if the length of
    /// `dynamic_offsets` is not correct for `layout`, or if any binding in any
    /// of `sets` is not initialized.
    ///
    /// If the value of the binding will not be used, create a dummy object of
    /// the appropriate type and bind it.
    ///
    /// The reference count of each member of `sets` is incremented.
    ///
    #[doc = crate::man_link!(vkCmdBindDescriptorSets)]
    pub fn bind_descriptor_sets(
        &mut self, pipeline_bind_point: PipelineBindPoint,
        layout: &'pool PipelineLayout, first_set: u32,
        sets: &[&'pool DescriptorSet], dynamic_offsets: &[u32],
    ) -> Result<()> {
        self.rec.bind_descriptor_sets(
            pipeline_bind_point,
            layout,
            first_set,
            sets,
            dynamic_offsets,
        )
    }
}
impl<'rec, 'pool> SecondaryCommandRecording<'rec, 'pool> {
    /// Returns [`Error::InvalidArgument`] if a member of `sets` is not compatible
    /// with the corresponding member of `layout`, if the length of
    /// `dynamic_offsets` is not correct for `layout`, or if any binding in any
    /// of `sets` is not initialized.
    ///
    /// If the value of the binding will not be used, create a dummy object of
    /// the appropriate type and bind it.
    ///
    /// The reference count of each member of `sets` is incremented.
    ///
    #[doc = crate::man_link!(vkCmdBindDescriptorSets)]
    pub fn bind_descriptor_sets(
        &mut self, pipeline_bind_point: PipelineBindPoint,
        layout: &'pool PipelineLayout, first_set: u32,
        sets: &[&'pool DescriptorSet], dynamic_offsets: &[u32],
    ) -> Result<()> {
        self.rec.bind_descriptor_sets(
            pipeline_bind_point,
            layout,
            first_set,
            sets,
            dynamic_offsets,
        )
    }
}

impl<'a> Bindings<'a> {
    fn bind_descriptor_sets(
        &mut self, layout: &'a PipelineLayout, begin: usize, sets: usize,
    ) {
        let end = begin + sets;
        let layouts = &layout.layouts()[0..end];
        let i = self
            .layout
            .iter()
            .zip(layouts.iter())
            .position(|(a, b)| a != b)
            .unwrap_or_else(|| self.layout.len().min(layouts.len()));
        if i < end {
            // Some bindings were invalidated
            self.layout.clear();
            self.layout.extend(layouts.iter().cloned());
            self.inited.resize(i, false);
            self.inited.resize(begin, false);
            self.inited.resize(end, true);
        } else {
            self.inited.resize(self.inited.len().max(end), false);
            self.inited[begin..end].fill(true);
        }
    }
}

impl<'rec, 'pool> CommandRecording<'rec, 'pool> {
    /// Returns [`Error::InvalidArgument`] if a member of `sets` is not compatible
    /// with the corresponding member of `layout`, if the length of
    /// `dynamic_offsets` is not correct for `layout`, or if any binding in any
    /// of `sets` is not [initialized](DescriptorSet::is_initialized).
    ///
    /// If the value of the binding will not be used, create a dummy object of
    /// the appropriate type and bind it.
    ///
    /// The reference count of each member of `sets` is incremented.
    ///
    #[doc = crate::man_link!(vkCmdBindDescriptorSets)]
    pub fn bind_descriptor_sets(
        &mut self, pipeline_bind_point: PipelineBindPoint,
        layout: &'pool PipelineLayout<'pool>, first_set: u32,
        sets: &[&'pool DescriptorSet], dynamic_offsets: &[u32],
    ) -> Result<()> {
        // Max binding is already checked by the layout
        let layouts = layout.layouts().iter().copied();
        if sets
            .iter()
            .map(|s| s.layout())
            .ne(layouts.skip(first_set as usize).take(sets.len()))
            || sets
                .iter()
                .map(|s| s.layout().num_dynamic_offsets())
                .sum::<u32>()
                != dynamic_offsets.len() as u32
            || sets.iter().any(|s| !s.is_initialized())
        {
            return Err(Error::InvalidArgument);
        }
        if pipeline_bind_point == PipelineBindPoint::GRAPHICS {
            self.graphics.bind_descriptor_sets(
                layout,
                first_set as usize,
                sets.len(),
            );
        } else {
            self.compute.bind_descriptor_sets(
                layout,
                first_set as usize,
                sets.len(),
            );
        }

        let sets =
            self.scratch.alloc_slice_fill_iter(sets.iter().map(|s| s.handle()));
        unsafe {
            (self.device.fun.cmd_bind_descriptor_sets)(
                self.buffer.handle_mut(),
                pipeline_bind_point,
                layout.handle(),
                first_set,
                sets.len() as u32,
                Array::from_slice(sets),
                dynamic_offsets.len() as u32,
                Array::from_slice(dynamic_offsets),
            )
        }

        Ok(())
    }
}

impl<'rec, 'pool> RenderPassRecording<'rec, 'pool> {
    /// Sets push constants. Returns [`Error::OutOfBounds`] if the data is out of
    /// bounds for push contants in `layout` or if `stage_flags` is incorrect.
    /// Returns [`Error::InvalidArgument`] if `data` is empty.
    #[doc = crate::man_link!(vkCmdPushConstants)]
    pub fn push_constants(
        &mut self, layout: &PipelineLayout, stage_flags: ShaderStageFlags,
        offset: u32, data: &[u8],
    ) -> Result<()> {
        self.rec.push_constants(layout, stage_flags, offset, data)
    }
}
impl<'rec, 'pool> SecondaryCommandRecording<'rec, 'pool> {
    /// Sets push constants. Returns [`Error::OutOfBounds`] if the data is out of
    /// bounds for push contants in `layout` or if `stage_flags` is incorrect.
    /// Returns [`Error::InvalidArgument`] if `data` is empty.
    #[doc = crate::man_link!(vkCmdPushConstants)]
    pub fn push_constants(
        &mut self, layout: &PipelineLayout, stage_flags: ShaderStageFlags,
        offset: u32, data: &[u8],
    ) -> Result<()> {
        self.rec.push_constants(layout, stage_flags, offset, data)
    }
}
impl<'rec, 'pool> CommandRecording<'rec, 'pool> {
    /// Sets push constants. Returns [`Error::OutOfBounds`] if the data is out of
    /// bounds for push contants in `layout` or if `stage_flags` is incorrect.
    /// Returns [`Error::InvalidArgument`] if `data` is empty.
    #[doc = crate::man_link!(vkCmdPushConstants)]
    pub fn push_constants(
        &mut self, layout: &PipelineLayout, stage_flags: ShaderStageFlags,
        offset: u32, data: &[u8],
    ) -> Result<()> {
        if !layout.bounds_check_push_constants(
            stage_flags,
            offset,
            data.len() as u32,
        ) {
            return Err(Error::OutOfBounds);
        }
        unsafe {
            (self.device.fun.cmd_push_constants)(
                self.buffer.handle_mut(),
                layout.handle(),
                stage_flags,
                offset,
                data.len() as u32,
                Array::from_slice(data).ok_or(Error::InvalidArgument)?,
            );
        }
        Ok(())
    }
}

#[cfg(test_disabled)]
mod test {
    use crate::vk;
    use std::sync::Arc;

    #[test]
    fn bounds_check() -> vk::Result<()> {
        let (dev, _) = crate::test_device()?;
        let buf = vk::BufferWithoutMemory::new(
            &dev,
            &vk::BufferCreateInfo { size: 1024, ..Default::default() },
        )?
        .allocate_memory(0)?;
        let img = vk::ImageWithoutMemory::new(
            &dev,
            &vk::ImageCreateInfo {
                extent: vk::Extent3D { width: 512, height: 512, depth: 1 },
                format: vk::Format::R8G8B8A8_SRGB,
                mip_levels: 10,
                ..Default::default()
            },
        )?
        .allocate_memory(0)?;
        let mut pool = vk::CommandPoolLifetime::new(&dev, 0)?;
        let mut rec = pool.begin();
        assert!(rec.fill_buffer(&buf, 100, Some(1024), 42).is_err());
        assert!(rec.fill_buffer(&buf, 2000, None, 42).is_err());
        assert!(rec
            .copy_buffer(
                &buf,
                &buf,
                &[vk::BufferCopy {
                    size: 1024,
                    src_offset: 0,
                    dst_offset: 100
                }]
            )
            .is_err());
        assert!(rec
            .copy_buffer(
                &buf,
                &buf,
                &[vk::BufferCopy {
                    size: 1024,
                    src_offset: 100,
                    dst_offset: 0
                }]
            )
            .is_err());
        assert!(rec
            .copy_buffer_to_image(
                &buf,
                &img,
                vk::ImageLayout::GENERAL,
                &[vk::BufferImageCopy {
                    image_offset: vk::Offset3D { x: 5, y: 0, z: 0 },
                    image_extent: vk::Extent3D {
                        width: 512,
                        height: 512,
                        depth: 1
                    },
                    ..Default::default()
                }]
            )
            .is_err());
        assert!(rec
            .copy_buffer_to_image(
                &buf,
                &img,
                vk::ImageLayout::GENERAL,
                &[vk::BufferImageCopy {
                    image_extent: vk::Extent3D {
                        width: 512,
                        height: 512,
                        depth: 1
                    },
                    image_subresource: vk::ImageSubresourceLayers {
                        layer_count: 4,
                        ..Default::default()
                    },
                    ..Default::default()
                }]
            )
            .is_err());
        assert!(rec
            .copy_buffer_to_image(
                &buf,
                &img,
                vk::ImageLayout::GENERAL,
                &[vk::BufferImageCopy {
                    image_extent: vk::Extent3D {
                        width: 512,
                        height: 512,
                        depth: 1
                    },
                    ..Default::default()
                }]
            )
            .is_err());

        Ok(())
    }

    const SPV: &[u32] = &[
        0x07230203, 0x00010000, 0x000d000a, 0x00000006, 0x00000000, 0x00020011,
        0x00000001, 0x0006000b, 0x00000001, 0x4c534c47, 0x6474732e, 0x3035342e,
        0x00000000, 0x0003000e, 0x00000000, 0x00000001, 0x0005000f, 0x00000005,
        0x00000004, 0x6e69616d, 0x00000000, 0x00060010, 0x00000004, 0x00000011,
        0x00000001, 0x00000001, 0x00000001, 0x00030003, 0x00000002, 0x000001c2,
        0x000a0004, 0x475f4c47, 0x4c474f4f, 0x70635f45, 0x74735f70, 0x5f656c79,
        0x656e696c, 0x7269645f, 0x69746365, 0x00006576, 0x00080004, 0x475f4c47,
        0x4c474f4f, 0x6e695f45, 0x64756c63, 0x69645f65, 0x74636572, 0x00657669,
        0x00040005, 0x00000004, 0x6e69616d, 0x00000000, 0x00020013, 0x00000002,
        0x00030021, 0x00000003, 0x00000002, 0x00050036, 0x00000002, 0x00000004,
        0x00000000, 0x00000003, 0x000200f8, 0x00000005, 0x000100fd, 0x00010038,
    ];

    #[test]
    fn descriptor_set_typecheck() -> vk::Result<()> {
        let (dev, _) = crate::test_device()?;
        let mut cmd_pool = vk::CommandPoolLifetime::new(&dev, 0)?;

        let ds_layout1 = vk::DescriptorSetLayout::new(
            &dev,
            vec![vk::DescriptorSetLayoutBinding {
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
                immutable_samplers: vec![vk::Sampler::new(
                    &dev,
                    &Default::default(),
                )?
                .handle()],
            }],
        )?;

        let ds_layout2 = vk::DescriptorSetLayout::new(
            &dev,
            vec![vk::DescriptorSetLayoutBinding {
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
                immutable_samplers: vec![],
            }],
        )?;

        let pipe_layout1 = vk::PipelineLayout::new(
            &dev,
            Default::default(),
            vec![&ds_layout1],
            vec![],
        )?;
        let pipe_layout2 = vk::PipelineLayout::new(
            &dev,
            Default::default(),
            vec![&ds_layout1, &ds_layout2],
            vec![],
        )?;
        let pipe_layout3 = vk::PipelineLayout::new(
            &dev,
            Default::default(),
            vec![&ds_layout2, &ds_layout2],
            vec![],
        )?;
        let pipe = vk::Pipeline::new_compute(
            vk::PipelineShaderStageCreateInfo::compute(
                &vk::ShaderModule::new(&dev, SPV).unwrap(),
            ),
            &pipe_layout2,
            None,
        )
        .unwrap();

        let buf = vk::BufferWithoutMemory::new(
            &dev,
            &vk::BufferCreateInfo {
                size: 1024,
                usage: vk::BufferUsageFlags::UNIFORM_BUFFER,
                ..Default::default()
            },
        )?
        .allocate_memory(0)?;
        let img = vk::ImageWithoutMemory::new(
            &dev,
            &vk::ImageCreateInfo {
                extent: vk::Extent3D { width: 512, height: 512, depth: 1 },
                format: vk::Format::R8G8B8A8_SRGB,
                mip_levels: 8,
                usage: vk::ImageUsageFlags::SAMPLED,
                ..Default::default()
            },
        )?
        .allocate_memory(0)?;
        let img = vk::ImageView::new(&img, &Default::default())?;
        let mut desc_pool = vk::DescriptorPool::new(
            &dev,
            4,
            &[
                vk::DescriptorPoolSize {
                    descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                    descriptor_count: 4,
                },
                vk::DescriptorPoolSize {
                    descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                    descriptor_count: 4,
                },
            ],
        )?;

        let desc_set1 =
            Arc::new(vk::DescriptorSet::new(&mut desc_pool, &ds_layout1)?);

        let cmd = cmd_pool.allocate()?;
        let mut rec = cmd_pool.begin(cmd)?;

        // Can't bind uninitialized set
        assert!(rec
            .bind_descriptor_sets(
                vk::PipelineBindPoint::COMPUTE,
                &pipe_layout1,
                0,
                &[&desc_set1],
                &[]
            )
            .is_err());

        let mut desc_set1 =
            vk::DescriptorSet::new(&mut desc_pool, &ds_layout1)?;
        let mut desc_set2 =
            vk::DescriptorSet::new(&mut desc_pool, &ds_layout2)?;

        vk::DescriptorSetUpdateBuilder::new(&dev)
            .begin()
            .dst_set(&mut desc_set1)
            .combined_image_samplers(0, 0, &[(&img, Default::default())])?
            .dst_set(&mut desc_set2)
            .uniform_buffers(
                0,
                0,
                &[vk::DescriptorBufferInfo {
                    buffer: &buf,
                    offset: 0,
                    range: Some(1024),
                }],
            )?
            .end();

        let desc_set1 = Arc::new(desc_set1);
        let desc_set2 = Arc::new(desc_set2);

        // Wrong layout
        assert!(rec
            .bind_descriptor_sets(
                vk::PipelineBindPoint::COMPUTE,
                &pipe_layout2,
                1,
                &[&desc_set1],
                &[]
            )
            .is_err());

        assert!(rec
            .bind_descriptor_sets(
                vk::PipelineBindPoint::COMPUTE,
                &pipe_layout1,
                0,
                &[&desc_set1],
                &[]
            )
            .is_ok());

        rec.bind_pipeline(&pipe);

        // Not everything bound
        assert!(rec.dispatch(1, 1, 1).is_err());

        // Invalidates earlier binding
        assert!(rec
            .bind_descriptor_sets(
                vk::PipelineBindPoint::COMPUTE,
                &pipe_layout3,
                1,
                &[&desc_set2],
                &[]
            )
            .is_ok());
        assert!(rec.dispatch(1, 1, 1).is_err());

        assert!(rec
            .bind_descriptor_sets(
                vk::PipelineBindPoint::COMPUTE,
                &pipe_layout1,
                0,
                &[&desc_set1],
                &[]
            )
            .is_ok());
        // Keeps earlier binding
        assert!(rec
            .bind_descriptor_sets(
                vk::PipelineBindPoint::COMPUTE,
                &pipe_layout2,
                1,
                &[&desc_set2],
                &[]
            )
            .is_ok());
        assert!(rec.dispatch(1, 1, 1).is_ok());

        // Invalidate
        assert!(rec
            .bind_descriptor_sets(
                vk::PipelineBindPoint::COMPUTE,
                &pipe_layout3,
                1,
                &[&desc_set2],
                &[]
            )
            .is_ok());

        assert!(rec
            .bind_descriptor_sets(
                vk::PipelineBindPoint::COMPUTE,
                &pipe_layout2,
                1,
                &[&desc_set2],
                &[]
            )
            .is_ok());
        // Keeps later binding
        assert!(rec
            .bind_descriptor_sets(
                vk::PipelineBindPoint::COMPUTE,
                &pipe_layout1,
                0,
                &[&desc_set1],
                &[]
            )
            .is_ok());
        assert!(rec.dispatch(1, 1, 1).is_ok());

        Ok(())
    }

    #[test]
    fn push_constant_check() -> vk::Result<()> {
        let (dev, _) = crate::test_device()?;
        let layout = vk::PipelineLayout::new(
            &dev,
            Default::default(),
            vec![],
            vec![
                vk::PushConstantRange {
                    stage_flags: vk::ShaderStageFlags::VERTEX,
                    offset: 0,
                    size: 16,
                },
                vk::PushConstantRange {
                    stage_flags: vk::ShaderStageFlags::FRAGMENT,
                    offset: 32,
                    size: 16,
                },
                vk::PushConstantRange {
                    stage_flags: vk::ShaderStageFlags::VERTEX,
                    offset: 48,
                    size: 16,
                },
            ],
        )?;
        let mut cmd_pool = vk::CommandPoolLifetime::new(&dev, 0)?;
        let buf = cmd_pool.allocate()?;
        let mut rec = cmd_pool.begin(buf)?;

        // Overflow
        assert!(rec
            .push_constants(
                &layout,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                u32::MAX,
                &[0; 48]
            )
            .is_err());
        // Touches void
        assert!(rec
            .push_constants(
                &layout,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                0,
                &[0; 48]
            )
            .is_err());
        // Stage missing
        assert!(rec
            .push_constants(
                &layout,
                vk::ShaderStageFlags::FRAGMENT,
                32,
                &[0; 32]
            )
            .is_err());
        // Ok
        assert!(rec
            .push_constants(
                &layout,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                32,
                &[0; 32]
            )
            .is_ok());

        Ok(())
    }
}
