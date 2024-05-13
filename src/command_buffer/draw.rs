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
use crate::subobject::Owner;
use crate::types::*;
use crate::vk::BufferUsageFlags;

use super::{
    Bindings, CommandRecording, ExternalRenderPassRecording,
    RenderPassRecording, SecondaryCommandBuffer, SecondaryCommandRecording,
};

impl<'a> RenderPassRecording<'a> {
    #[doc = crate::man_link!(vkCmdSetViewport)]
    pub fn set_viewport(&mut self, viewport: &Viewport) {
        self.rec.set_viewport(viewport)
    }
}
impl<'a> SecondaryCommandRecording<'a> {
    #[doc = crate::man_link!(vkCmdSetViewport)]
    pub fn set_viewport(&mut self, viewport: &Viewport) {
        self.rec.set_viewport(viewport)
    }
}
impl<'a> CommandRecording<'a> {
    #[doc = crate::man_link!(vkCmdSetViewport)]
    pub fn set_viewport(&mut self, viewport: &Viewport) {
        unsafe {
            (self.pool.device.fun.cmd_set_viewport)(
                self.buffer.handle.borrow_mut(),
                0,
                1,
                std::array::from_ref(viewport).into(),
            )
        }
    }
}

impl<'a> RenderPassRecording<'a> {
    #[doc = crate::man_link!(vkCmdSetScissor)]
    pub fn set_scissor(&mut self, scissor: &Rect2D) {
        self.rec.set_scissor(scissor)
    }
}
impl<'a> SecondaryCommandRecording<'a> {
    #[doc = crate::man_link!(vkCmdSetScissor)]
    pub fn set_scissor(&mut self, scissor: &Rect2D) {
        self.rec.set_scissor(scissor)
    }
}
impl<'a> CommandRecording<'a> {
    #[doc = crate::man_link!(vkCmdSetScissor)]
    pub fn set_scissor(&mut self, scissor: &Rect2D) {
        unsafe {
            (self.pool.device.fun.cmd_set_scissor)(
                self.buffer.handle.borrow_mut(),
                0,
                1,
                std::array::from_ref(scissor).into(),
            )
        }
    }
}

impl<'a> Bindings<'a> {
    fn check(&self) -> Result<()> {
        if let Some(pipeline) = self.pipeline.as_ref() {
            let layouts = &pipeline.layout().layouts();
            if self.layout.get(0..layouts.len()) == Some(layouts)
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

impl<'a> RenderPassRecording<'a> {
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
        &mut self, buffer: &Arc<Buffer>, offset: u64, draw_count: u32,
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
        &mut self, buffer: &Arc<Buffer>, offset: u64, draw_count: u32,
        stride: u32,
    ) -> Result<()> {
        self.rec.graphics.check_render_pass(&self.pass, self.subpass)?;
        self.rec.draw_indexed_indirect(buffer, offset, draw_count, stride)
    }
}
impl<'a> SecondaryCommandRecording<'a> {
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
        &mut self, buffer: &Arc<Buffer>, offset: u64, draw_count: u32,
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
        &mut self, buffer: &Arc<Buffer>, offset: u64, draw_count: u32,
        stride: u32,
    ) -> Result<()> {
        self.rec.graphics.check_render_pass(&self.pass, self.subpass)?;
        self.rec.draw_indexed_indirect(buffer, offset, draw_count, stride)
    }
}

fn bounds_check_n(
    count: u32, size: u32, mut stride: u32, buf: &Arc<Buffer>, offset: u64,
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

impl<'a> CommandRecording<'a> {
    fn draw(
        &mut self, vertex_count: u32, instance_count: u32, first_vertex: u32,
        first_instance: u32,
    ) -> Result<()> {
        self.graphics.check()?;
        unsafe {
            (self.pool.device.fun.cmd_draw)(
                self.buffer.handle.borrow_mut(),
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            )
        }
        Ok(())
    }
    fn draw_indirect(
        &mut self, buffer: &Arc<Buffer>, offset: u64, draw_count: u32,
        stride: u32,
    ) -> Result<()> {
        if !buffer.usage().contains(BufferUsageFlags::INDIRECT_BUFFER) {
            return Err(Error::InvalidArgument);
        }
        bounds_check_n(draw_count, 16, stride, buffer, offset)?;
        self.graphics.check()?;
        self.add_resource(buffer.clone());
        unsafe {
            (self.pool.device.fun.cmd_draw_indirect)(
                self.buffer.handle.borrow_mut(),
                buffer.borrow(),
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
            (self.pool.device.fun.cmd_draw_indexed)(
                self.buffer.handle.borrow_mut(),
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
        &mut self, buffer: &Arc<Buffer>, offset: u64, draw_count: u32,
        stride: u32,
    ) -> Result<()> {
        bounds_check_n(draw_count, 20, stride, buffer, offset)?;
        if !buffer.usage().contains(BufferUsageFlags::INDIRECT_BUFFER) {
            return Err(Error::InvalidArgument);
        }
        self.graphics.check()?;
        self.add_resource(buffer.clone());
        unsafe {
            (self.pool.device.fun.cmd_draw_indexed_indirect)(
                self.buffer.handle.borrow_mut(),
                buffer.borrow(),
                offset,
                draw_count,
                stride,
            )
        }
        Ok(())
    }
}

impl<'a> CommandRecording<'a> {
    #[doc = crate::man_link!(vkCmdDispatch)]
    pub fn dispatch(
        &mut self, group_count_x: u32, group_count_y: u32, group_count_z: u32,
    ) -> Result<()> {
        self.compute.check()?;
        unsafe {
            (self.pool.device.fun.cmd_dispatch)(
                self.buffer.handle.borrow_mut(),
                group_count_x,
                group_count_y,
                group_count_z,
            );
        }
        Ok(())
    }
    #[doc = crate::man_link!(vkCmdDispatchIndirect)]
    pub fn dispatch_indirect(
        &mut self, buffer: &Arc<Buffer>, offset: u64,
    ) -> Result<()> {
        self.compute.check()?;
        self.add_resource(buffer.clone());
        unsafe {
            (self.pool.device.fun.cmd_dispatch_indirect)(
                self.buffer.handle.borrow_mut(),
                buffer.borrow(),
                offset,
            );
        }
        Ok(())
    }
}

impl<'a> ExternalRenderPassRecording<'a> {
    /// Returns [Error::InvalidArgument] if 'commands' is empty, if a member of
    /// 'commands' is not in the executable state, or if a member of 'commands'
    /// is not compatible with the current pass and subpass. Returns
    /// [Error::SynchronizationError] if a member of 'commands' is currently
    /// recorded to another command buffer.
    ///
    /// If a command was recorded from another pool, increments the reference
    /// count of that pool. That is, this pool must be reset before the other
    /// one can be. Note that this **does not** check for cycles of length
    /// greater than one: Adding a secondary command buffer from pool A to a
    /// primary from pool B, and a secondary from pool B to a primary from pool
    /// A will leak both pools.
    ///
    #[doc = crate::man_link!(vkCmdExecuteCommands)]
    pub fn execute_commands(
        &mut self, commands: &mut [&mut SecondaryCommandBuffer],
    ) -> Result<()> {
        let mut resources = bumpalo::vec![in self.rec.scratch];
        let mut handles = bumpalo::vec![in self.rec.scratch];
        for command in commands.iter_mut() {
            if !self.pass.compatible(command.pass.as_deref().unwrap())
                || self.subpass != command.subpass
            {
                return Err(Error::InvalidArgument);
            }
            // Check that the buffer is recorded.
            let res = command.lock_resources().ok_or(Error::InvalidArgument)?;
            // Require that this pool be reset before the other pool.
            if !Owner::ptr_eq(self.rec.pool, &command.buf.pool) {
                resources.push(res as Arc<_>);
            }
            // Check that the buffer is not in use.
            handles.push(command.borrow_mut()?);
        }

        unsafe {
            (self.rec.pool.device.fun.cmd_execute_commands)(
                self.rec.buffer.handle.borrow_mut(),
                handles.len() as u32,
                Array::from_slice(&handles).ok_or(Error::InvalidArgument)?,
            )
        }

        drop(handles);
        self.rec.pool.resources.extend(resources);
        for command in commands {
            // Prevent this buffer from being reused.
            self.rec.pool.resources.push(command.lock_self());
        }
        Ok(())
    }
}
