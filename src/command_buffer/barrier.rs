// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![allow(clippy::too_many_arguments)]
use crate::enums::*;
use crate::ffi::Array;
use crate::image::Image;
use crate::types::*;

use super::{CommandRecording, RenderPassRecording, SecondaryCommandRecording};

impl<'rec, 'pool> RenderPassRecording<'rec, 'pool> {
    #[doc = crate::man_link!(vkCmdPipelineBarrier)]
    pub fn pipeline_barrier(
        &mut self, src_stage_mask: PipelineStageFlags,
        dst_stage_mask: PipelineStageFlags, dependency_flags: DependencyFlags,
        memory_barriers: &[MemoryBarrier],
        buffer_memory_barriers: &[BufferMemoryBarrier<'pool>],
        image_memory_barriers: &[ImageMemoryBarrier<'pool>],
    ) {
        self.rec.pipeline_barrier(
            src_stage_mask,
            dst_stage_mask,
            dependency_flags,
            memory_barriers,
            buffer_memory_barriers,
            image_memory_barriers,
        )
    }
    /// A shortcut for simple memory barriers
    #[doc = crate::man_link!(vkCmdPipelineBarrier)]
    pub fn memory_barrier(
        &mut self, src_stage_mask: PipelineStageFlags,
        dst_stage_mask: PipelineStageFlags, src_access_mask: AccessFlags,
        dst_access_mask: AccessFlags,
    ) {
        self.rec.memory_barrier(
            src_stage_mask,
            dst_stage_mask,
            src_access_mask,
            dst_access_mask,
        )
    }
    /// A shortcut for simple image barriers
    #[doc = crate::man_link!(vkCmdPipelineBarrier)]
    pub fn image_barrier(
        &mut self, image: &'pool Image, src_stage_mask: PipelineStageFlags,
        dst_stage_mask: PipelineStageFlags, src_access_mask: AccessFlags,
        dst_access_mask: AccessFlags, old_layout: ImageLayout,
        new_layout: ImageLayout,
    ) {
        self.rec.image_barrier(
            image,
            src_stage_mask,
            dst_stage_mask,
            src_access_mask,
            dst_access_mask,
            old_layout,
            new_layout,
        )
    }
}

impl<'rec, 'pool> SecondaryCommandRecording<'rec, 'pool> {
    #[doc = crate::man_link!(vkCmdPipelineBarrier)]
    pub fn pipeline_barrier(
        &mut self, src_stage_mask: PipelineStageFlags,
        dst_stage_mask: PipelineStageFlags, dependency_flags: DependencyFlags,
        memory_barriers: &[MemoryBarrier],
        buffer_memory_barriers: &[BufferMemoryBarrier<'pool>],
        image_memory_barriers: &[ImageMemoryBarrier<'pool>],
    ) {
        self.rec.pipeline_barrier(
            src_stage_mask,
            dst_stage_mask,
            dependency_flags,
            memory_barriers,
            buffer_memory_barriers,
            image_memory_barriers,
        )
    }
    /// A shortcut for simple memory barriers.
    #[doc = crate::man_link!(vkCmdPipelineBarrier)]
    pub fn memory_barrier(
        &mut self, src_stage_mask: PipelineStageFlags,
        dst_stage_mask: PipelineStageFlags, src_access_mask: AccessFlags,
        dst_access_mask: AccessFlags,
    ) {
        self.rec.memory_barrier(
            src_stage_mask,
            dst_stage_mask,
            src_access_mask,
            dst_access_mask,
        )
    }
    /// A shortcut for simple image barriers.
    #[doc = crate::man_link!(vkCmdPipelineBarrier)]
    pub fn image_barrier(
        &mut self, image: &'pool Image, src_stage_mask: PipelineStageFlags,
        dst_stage_mask: PipelineStageFlags, src_access_mask: AccessFlags,
        dst_access_mask: AccessFlags, old_layout: ImageLayout,
        new_layout: ImageLayout,
    ) {
        self.rec.image_barrier(
            image,
            src_stage_mask,
            dst_stage_mask,
            src_access_mask,
            dst_access_mask,
            old_layout,
            new_layout,
        )
    }
}

impl<'rec, 'pool> CommandRecording<'rec, 'pool> {
    #[doc = crate::man_link!(vkCmdPipelineBarrier)]
    pub fn pipeline_barrier(
        &mut self, src_stage_mask: PipelineStageFlags,
        dst_stage_mask: PipelineStageFlags, dependency_flags: DependencyFlags,
        memory_barriers: &[MemoryBarrier],
        buffer_memory_barriers: &[BufferMemoryBarrier<'pool>],
        image_memory_barriers: &[ImageMemoryBarrier<'pool>],
    ) {
        unsafe {
            (self.device.fun.cmd_pipeline_barrier)(
                self.buffer.handle_mut(),
                src_stage_mask,
                dst_stage_mask,
                dependency_flags,
                memory_barriers.len() as u32,
                Array::from_slice(memory_barriers),
                buffer_memory_barriers.len() as u32,
                Array::from_slice(buffer_memory_barriers),
                image_memory_barriers.len() as u32,
                Array::from_slice(image_memory_barriers),
            )
        }
    }

    /// A shortcut for simple memory barriers.
    #[doc = crate::man_link!(vkCmdPipelineBarrier)]
    pub fn memory_barrier(
        &mut self, src_stage_mask: PipelineStageFlags,
        dst_stage_mask: PipelineStageFlags, src_access_mask: AccessFlags,
        dst_access_mask: AccessFlags,
    ) {
        unsafe {
            (self.device.fun.cmd_pipeline_barrier)(
                self.buffer.handle_mut(),
                src_stage_mask,
                dst_stage_mask,
                Default::default(),
                1,
                Some(Array::from(&[MemoryBarrier {
                    src_access_mask,
                    dst_access_mask,
                    ..Default::default()
                }])),
                0,
                None,
                0,
                None,
            )
        }
    }

    /// A shortcut for simple image barriers.
    #[doc = crate::man_link!(vkCmdPipelineBarrier)]
    pub fn image_barrier(
        &mut self, image: &'pool Image, src_stage_mask: PipelineStageFlags,
        dst_stage_mask: PipelineStageFlags, src_access_mask: AccessFlags,
        dst_access_mask: AccessFlags, old_layout: ImageLayout,
        new_layout: ImageLayout,
    ) {
        unsafe {
            let barrier = ImageMemoryBarrier {
                stype: Default::default(),
                next: Default::default(),
                src_access_mask,
                dst_access_mask,
                old_layout,
                new_layout,
                src_queue_family_index: Default::default(),
                dst_queue_family_index: Default::default(),
                image: image.borrow(),
                subresource_range: Default::default(),
            };
            (self.device.fun.cmd_pipeline_barrier)(
                self.buffer.handle_mut(),
                src_stage_mask,
                dst_stage_mask,
                Default::default(),
                0,
                None,
                0,
                None,
                1,
                Array::from_slice(&[barrier]),
            )
        }
    }
}
