// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::buffer::Buffer;
use crate::error::{Error, Result};
use crate::ffi::Array;
use crate::render_pass::RenderPass;
use crate::types::*;
use crate::vk::BufferUsageFlags;

use super::{
    Bindings, CommandRecording, ExternalRenderPassRecording,
    RenderPassRecording, SecondaryCommandBuffer, SecondaryCommandRecording,
};

impl<'rec, 'pool> RenderPassRecording<'rec, 'pool> {
    #[doc = crate::man_link!(vkCmdSetViewport)]
    pub fn set_viewport(&mut self, viewport: &Viewport) {
        self.rec.set_viewport(viewport)
    }
}
impl<'rec, 'pool> SecondaryCommandRecording<'rec, 'pool> {
    #[doc = crate::man_link!(vkCmdSetViewport)]
    pub fn set_viewport(&mut self, viewport: &Viewport) {
        self.rec.set_viewport(viewport)
    }
}
impl<'rec, 'pool> CommandRecording<'rec, 'pool> {
    #[doc = crate::man_link!(vkCmdSetViewport)]
    pub fn set_viewport(&mut self, viewport: &Viewport) {
        unsafe {
            (self.device.fun().cmd_set_viewport)(
                self.buffer.handle_mut(),
                0,
                1,
                std::array::from_ref(viewport).into(),
            )
        }
    }
}

impl<'rec, 'pool> RenderPassRecording<'rec, 'pool> {
    #[doc = crate::man_link!(vkCmdSetScissor)]
    pub fn set_scissor(&mut self, scissor: &Rect2D) {
        self.rec.set_scissor(scissor)
    }
}
impl<'rec, 'pool> SecondaryCommandRecording<'rec, 'pool> {
    #[doc = crate::man_link!(vkCmdSetScissor)]
    pub fn set_scissor(&mut self, scissor: &Rect2D) {
        self.rec.set_scissor(scissor)
    }
}
impl<'rec, 'pool> CommandRecording<'rec, 'pool> {
    #[doc = crate::man_link!(vkCmdSetScissor)]
    pub fn set_scissor(&mut self, scissor: &Rect2D) {
        unsafe {
            (self.device.fun().cmd_set_scissor)(
                self.buffer.handle_mut(),
                0,
                1,
                std::array::from_ref(scissor).into(),
            )
        }
    }
}

impl<'rec> Bindings<'rec> {
    fn check(&self) -> Result<()> {
        if let Some(pipeline) = self.pipeline.as_ref() {
            // Is this not just checked by inited?
            let layouts = pipeline.layout().layouts();
            if self.layout.len() >= layouts.len()
                && self.layout[0..layouts.len()]
                    .iter()
                    .copied()
                    .eq(layouts.iter())
                && self.inited.iter().take_while(|b| **b).count()
                    >= layouts.len()
            {
                return Ok(());
            }
        }
        Err(Error::InvalidState)
    }
    fn check_render_pass(&self, pass: &RenderPass, subpass: u32) -> Result<()> {
        if let Some(pipeline) = self.pipeline.as_ref() {
            if pipeline.is_compatible_with(pass, subpass) {
                return Ok(());
            }
        }
        Err(Error::InvalidState)
    }
}

macro_rules! draw_state {
    () => {
        "Returns [`Error::InvalidState`] if the bound pipeline is not compatible
        with the current render pass and subpass, if the bound descriptor sets
        and bound graphics pipeline do not have a compatible layout, or if a
        descriptor set mentioned in the pipeline's layout is not bound."
    }
}

impl<'rec, 'pool> RenderPassRecording<'rec, 'pool> {
    #[doc = draw_state!()]
    ///
    #[doc = crate::man_link!(vkCmdDraw)]
    pub fn draw(
        &mut self, vertex_count: u32, instance_count: u32, first_vertex: u32,
        first_instance: u32,
    ) -> Result<()> {
        self.rec.graphics.check_render_pass(&self.pass, self.subpass)?;
        self.rec.draw(
            vertex_count,
            instance_count,
            first_vertex,
            first_instance,
        )
    }
    #[doc = draw_state!()]
    ///
    /// The reference count of `buffer` is incremented.
    ///
    #[doc = crate::man_link!(vkCmdDrawIndirect)]
    pub fn draw_indirect(
        &mut self, buffer: &'pool Buffer, offset: u64, draw_count: u32,
        stride: u32,
    ) -> Result<()> {
        self.rec.graphics.check_render_pass(&self.pass, self.subpass)?;
        self.rec.draw_indirect(buffer, offset, draw_count, stride)
    }
    #[doc = draw_state!()]
    ///
    #[doc = crate::man_link!(vkCmdDrawIndexed)]
    pub fn draw_indexed(
        &mut self, index_count: u32, instance_count: u32, first_index: u32,
        vertex_offset: i32, first_instance: u32,
    ) -> Result<()> {
        self.rec.graphics.check_render_pass(&self.pass, self.subpass)?;
        self.rec.draw_indexed(
            index_count,
            instance_count,
            first_index,
            vertex_offset,
            first_instance,
        )
    }
    #[doc = draw_state!()]
    ///
    /// The reference count of `buffer` is incremented.
    ///
    #[doc = crate::man_link!(vkCmdDrawIndexedIndirect)]
    pub fn draw_indexed_indirect(
        &mut self, buffer: &'pool Buffer, offset: u64, draw_count: u32,
        stride: u32,
    ) -> Result<()> {
        self.rec.graphics.check_render_pass(&self.pass, self.subpass)?;
        self.rec.draw_indexed_indirect(buffer, offset, draw_count, stride)
    }
}
impl<'rec, 'pool> SecondaryCommandRecording<'rec, 'pool> {
    #[doc = draw_state!()]
    ///
    #[doc = crate::man_link!(vkCmdDraw)]
    pub fn draw(
        &mut self, vertex_count: u32, instance_count: u32, first_vertex: u32,
        first_instance: u32,
    ) -> Result<()> {
        self.rec.graphics.check_render_pass(&self.pass, self.subpass)?;
        self.rec.draw(
            vertex_count,
            instance_count,
            first_vertex,
            first_instance,
        )
    }
    #[doc = draw_state!()]
    ///
    /// The reference count of `buffer` is incremented.
    ///
    #[doc = crate::man_link!(vkCmdDrawIndirect)]
    pub fn draw_indirect(
        &mut self, buffer: &'pool Buffer, offset: u64, draw_count: u32,
        stride: u32,
    ) -> Result<()> {
        self.rec.graphics.check_render_pass(&self.pass, self.subpass)?;
        self.rec.draw_indirect(buffer, offset, draw_count, stride)
    }
    #[doc = draw_state!()]
    ///
    #[doc = crate::man_link!(vkCmdDrawIndexed)]
    pub fn draw_indexed(
        &mut self, index_count: u32, instance_count: u32, first_index: u32,
        vertex_offset: i32, first_instance: u32,
    ) -> Result<()> {
        self.rec.graphics.check_render_pass(&self.pass, self.subpass)?;
        self.rec.draw_indexed(
            index_count,
            instance_count,
            first_index,
            vertex_offset,
            first_instance,
        )
    }
    #[doc = draw_state!()]
    ///
    /// The reference count of `buffer` is incremented.
    ///
    #[doc = crate::man_link!(vkCmdDrawIndexedIndirect)]
    pub fn draw_indexed_indirect(
        &mut self, buffer: &'pool Buffer, offset: u64, draw_count: u32,
        stride: u32,
    ) -> Result<()> {
        self.rec.graphics.check_render_pass(&self.pass, self.subpass)?;
        self.rec.draw_indexed_indirect(buffer, offset, draw_count, stride)
    }
}

fn bounds_check_n(
    count: u32, size: u32, mut stride: u32, buf: &Buffer, offset: u64,
) -> Result<()> {
    if count < 2 {
        stride = size;
    }
    if stride < size || offset & 3 != 0 {
        return Err(Error::InvalidArgument);
    }
    let len =
        (count as u64).checked_mul(stride as u64).ok_or(Error::OutOfBounds)?;
    if !buf.bounds_check(offset, len) {
        return Err(Error::OutOfBounds);
    }
    Ok(())
}

impl<'rec, 'pool> CommandRecording<'rec, 'pool> {
    fn draw(
        &mut self, vertex_count: u32, instance_count: u32, first_vertex: u32,
        first_instance: u32,
    ) -> Result<()> {
        self.graphics.check()?;
        unsafe {
            (self.device.fun().cmd_draw)(
                self.buffer.handle_mut(),
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            )
        }
        Ok(())
    }
    fn draw_indirect(
        &mut self, buffer: &'pool Buffer, offset: u64, draw_count: u32,
        stride: u32,
    ) -> Result<()> {
        if !buffer.usage().contains(BufferUsageFlags::INDIRECT_BUFFER) {
            return Err(Error::InvalidArgument);
        }
        bounds_check_n(draw_count, 16, stride, buffer, offset)?;
        self.graphics.check()?;
        unsafe {
            (self.device.fun().cmd_draw_indirect)(
                self.buffer.handle_mut(),
                buffer.handle(),
                offset,
                draw_count,
                stride,
            )
        }
        Ok(())
    }
    fn draw_indexed(
        &mut self, index_count: u32, instance_count: u32, first_index: u32,
        vertex_offset: i32, first_instance: u32,
    ) -> Result<()> {
        self.graphics.check()?;
        unsafe {
            (self.device.fun().cmd_draw_indexed)(
                self.buffer.handle_mut(),
                index_count,
                instance_count,
                first_index,
                vertex_offset,
                first_instance,
            )
        }
        Ok(())
    }
    fn draw_indexed_indirect(
        &mut self, buffer: &'pool Buffer, offset: u64, draw_count: u32,
        stride: u32,
    ) -> Result<()> {
        bounds_check_n(draw_count, 20, stride, buffer, offset)?;
        if !buffer.usage().contains(BufferUsageFlags::INDIRECT_BUFFER) {
            return Err(Error::InvalidArgument);
        }
        self.graphics.check()?;
        unsafe {
            (self.device.fun().cmd_draw_indexed_indirect)(
                self.buffer.handle_mut(),
                buffer.handle(),
                offset,
                draw_count,
                stride,
            )
        }
        Ok(())
    }
}

impl<'rec, 'pool> CommandRecording<'rec, 'pool> {
    #[doc = crate::man_link!(vkCmdDispatch)]
    pub fn dispatch(
        &mut self, group_count_x: u32, group_count_y: u32, group_count_z: u32,
    ) -> Result<()> {
        self.compute.check()?;
        unsafe {
            (self.device.fun().cmd_dispatch)(
                self.buffer.handle_mut(),
                group_count_x,
                group_count_y,
                group_count_z,
            );
        }
        Ok(())
    }
    #[doc = crate::man_link!(vkCmdDispatchIndirect)]
    pub fn dispatch_indirect(
        &mut self, buffer: &'pool Buffer, offset: u64,
    ) -> Result<()> {
        self.compute.check()?;
        unsafe {
            (self.device.fun().cmd_dispatch_indirect)(
                self.buffer.handle_mut(),
                buffer.handle(),
                offset,
            );
        }
        Ok(())
    }
}

impl<'rec, 'pool> ExternalRenderPassRecording<'rec, 'pool> {
    /// Returns [Error::InvalidArgument] if 'commands' is empty, if a member of
    /// 'commands' is not in the executable state, or if a member of 'commands'
    /// is not compatible with the current pass and subpass. Returns
    /// [Error::SynchronizationError] if a member of 'commands' is currently
    /// recorded to another command buffer.
    ///
    #[doc = crate::man_link!(vkCmdExecuteCommands)]
    pub fn execute_commands(
        &mut self, commands: &mut [&'pool mut SecondaryCommandBuffer],
    ) -> Result<()> {
        let mut handles = bumpalo::vec![in self.rec.scratch];
        for command in commands.iter_mut() {
            if !self.pass.compatible(command.pass.as_deref().unwrap())
                || self.subpass != command.subpass
            {
                return Err(Error::InvalidArgument);
            }
            handles.push(command.borrow_mut());
        }

        unsafe {
            (self.rec.device.fun().cmd_execute_commands)(
                self.rec.buffer.handle_mut(),
                handles.len() as u32,
                Array::from_slice(&handles).ok_or(Error::InvalidArgument)?,
            )
        }

        Ok(())
    }
}
