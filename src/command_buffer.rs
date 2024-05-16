// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::cell::RefCell;
use std::fmt::Debug;

use crate::descriptor_set::DescriptorSetLayout;
use crate::device::Device;
use crate::enums::*;
use crate::error::{Error, Result};
use crate::framebuffer::Framebuffer;
use crate::pipeline::Pipeline;
use crate::render_pass::RenderPass;
use crate::types::*;
use crate::vk::ArrayMut;

mod transfer;
pub mod barrier;
mod bind;
mod draw;

/// A command pool.
#[derive(Debug)]
pub struct CommandPool<'d> {
    handle: Handle<VkCommandPool>,
    buffers: RefCell<Vec<Handle<VkCommandBuffer>>>,
    free_buffers: RefCell<Vec<usize>>,
    device: &'d Device<'d>,
    scratch: bumpalo::Bump,
}

/// A primary command buffer.
#[derive(Debug)]
pub struct CommandBuffer<'a>(Mut<'a, VkCommandBuffer>);

/// A secondary command buffer.
///
///
/// Create with [`CommandPool::allocate_secondary`]
#[derive(Debug)]
pub struct SecondaryCommandBuffer<'a> {
    buf: Mut<'a, VkCommandBuffer>,
    pass: Option<&'a RenderPass<'a>>,
    subpass: u32,
}

#[derive(Debug)]
struct Bindings<'a> {
    layout: bumpalo::collections::Vec<'a, &'a DescriptorSetLayout<'a>>,
    inited: bumpalo::collections::Vec<'a, bool>,
    pipeline: Option<&'a Pipeline<'a>>,
}

// TODO: Use deref to make this less repetitive?

/// An in-progress command buffer recording, outside of a render pass.
#[derive(Debug)]
pub struct CommandRecording<'a> {
    pool: &'a CommandPool<'a>,
    scratch: &'a bumpalo::Bump,
    graphics: Bindings<'a>,
    compute: Bindings<'a>,
    buffer: CommandBuffer<'a>,
}

/// An in-progress command buffer recording, inside a render pass.
#[must_use = "Record render pass commands on this object"]
#[derive(Debug)]
pub struct RenderPassRecording<'a> {
    rec: CommandRecording<'a>,
    pass: &'a RenderPass<'a>,
    subpass: u32,
}

/// An in-progress command buffer recording, inside a render pass whose contents
/// is provided with secondary command buffers.
#[must_use = "Record secondary command buffers on this object"]
#[derive(Debug)]
pub struct ExternalRenderPassRecording<'a> {
    rec: CommandRecording<'a>,
    pass: Arc<RenderPass<'a>>,
    subpass: u32,
}

/// An in-progress secondary command buffer recording, inside a render pass.
#[derive(Debug)]
pub struct SecondaryCommandRecording<'a> {
    rec: CommandRecording<'a>,
    pass: Arc<RenderPass<'a>>,
    subpass: u32,
}

impl<'d> CommandPool<'d> {
    /// Create a command pool. The pool is not transient, not protected, and its
    /// buffers cannot be individually reset.
    #[doc = crate::man_link!(vkCreateCommandPool)]
    pub fn new(device: &'d Device, queue_family_index: u32) -> Result<Self> {
        if !device.has_queue(queue_family_index, 1) {
            return Err(Error::OutOfBounds);
        }
        let mut handle = None;
        unsafe {
            (device.fun.create_command_pool)(
                device.handle(),
                &CommandPoolCreateInfo {
                    queue_family_index,
                    ..Default::default()
                },
                None,
                &mut handle,
            )?;
        }
        let handle = handle.unwrap();

        Ok(CommandPool {
            handle,
            buffers: Default::default(),
            free_buffers: Default::default(),
            device,
            scratch: bumpalo::Bump::new(),
        })
    }
}

impl Drop for CommandPool<'_> {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun.destroy_command_pool)(
                self.device.handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

impl CommandPool<'_> {
    /// Borrows the inner Vulkan handle.
    pub fn handle_mut(&mut self) -> Mut<VkCommandPool> {
        self.handle.borrow_mut()
    }

    fn len(&self) -> usize {
        self.buffers.borrow().len()
    }
    fn reserve(&self, additional: u32) {
        let mut vec = self.buffers.borrow_mut();
        vec.reserve(additional as usize);
        let old_len = vec.len();
        let new_len = old_len + additional as usize;
        unsafe {
            (self.device.fun.allocate_command_buffers)(
                self.device.handle(),
                &CommandBufferAllocateInfo {
                    stype: Default::default(),
                    next: Default::default(),
                    pool: self.handle.borrow_mut_unchecked(),
                    level: CommandBufferLevel::PRIMARY,
                    count: additional,
                },
                ArrayMut::from_slice(vec.spare_capacity_mut()).unwrap(),
            )
            .unwrap();
            vec.set_len(new_len);
        }
        self.free_buffers.borrow_mut().extend(old_len..new_len);
    }

    /// Resets the pool and adds all command buffers to the free list.
    #[doc = crate::man_link!(vkResetCommandPool)]
    pub fn reset(&mut self, flags: CommandPoolResetFlags) -> Result<()> {
        unsafe {
            (self.device.fun.reset_command_pool)(
                self.device.handle(),
                self.handle.borrow_mut(),
                flags,
            )?;
        }
        self.free_buffers.borrow_mut().clear();
        self.free_buffers.borrow_mut().extend(0..self.len());
        self.scratch.reset();
        Ok(())
    }

    /// Begin a command buffer, allocating a new one if one is not available on the free list.
    #[doc = crate::man_link!(vkAllocateCommandBuffers)]
    #[doc = crate::man_link!(vkBeginCommandBuffer)]
    pub fn begin<'a>(&'a self) -> CommandRecording<'a> {
        if self.free_buffers.borrow().is_empty() {
            self.reserve(1)
        }
        let buffer = self.free_buffers.borrow_mut().pop().unwrap();
        // Safety: Moving the Handle<> doesn't actually invalidate the reference.
        let mut buffer: Mut<'a, VkCommandBuffer> = unsafe {
            self.buffers.borrow_mut()[buffer]
                .borrow_mut()
                .reborrow_mut_unchecked()
        };
        unsafe {
            (self.device.fun.begin_command_buffer)(
                buffer.reborrow_mut(),
                &Default::default(),
            )
            .unwrap();
        }
        CommandRecording {
            pool: self,
            graphics: Bindings::new(&self.scratch),
            compute: Bindings::new(&self.scratch),
            scratch: &self.scratch,
            buffer: CommandBuffer(buffer),
        }
    }
    /*
    /// Returns [`Error::InvalidArgument`] if the buffer does not belong to this
    /// pool or is in the executable state. Returns
    /// [`Error::SynchronizationError`] if the buffer is in the pending state.
    #[doc = crate::man_link!(vkBeginCommandBuffer)]
    pub fn begin_secondary<'a>(
        &'a mut self, buffer: SecondaryCommandBuffer,
        render_pass: &Arc<RenderPass>, subpass: u32,
    ) -> ResultAndSelf<SecondaryCommandRecording<'a>, SecondaryCommandBuffer>
    {
        if subpass >= render_pass.num_subpasses() {
            return Err(ErrorAndSelf(Error::InvalidArgument, buffer));
        }
        if !Owner::ptr_eq(&self.res, &buffer.buf.pool)
            // In executable state
            || buffer.lock_resources().is_some()
        {
            return Err(ErrorAndSelf(Error::InvalidArgument, buffer));
        }
        // In pending state
        let mut inner = Owner::from_arc(buffer.buf).map_err(|arc| {
            ErrorAndSelf(
                Error::SynchronizationError,
                SecondaryCommandBuffer { buf: arc, pass: None, subpass: 0 },
            )
        })?;
        unsafe {
            if let Err(err) = (self.res.device.fun.begin_command_buffer)(
                inner.handle.borrow_mut(),
                &CommandBufferBeginInfo {
                    flags: CommandBufferUsageFlags::RENDER_PASS_CONTINUE,
                    inheritance_info: Some(&CommandBufferInheritanceInfo {
                        stype: Default::default(),
                        next: Default::default(),
                        render_pass: render_pass.handle(),
                        subpass,
                        framebuffer: Default::default(),
                        occlusion_query_enable: Default::default(),
                        query_flags: Default::default(),
                        pipeline_statistics: Default::default(),
                    }),
                    ..Default::default()
                },
            ) {
                return Err(ErrorAndSelf(
                    err.into(),
                    SecondaryCommandBuffer {
                        buf: Owner::into_arc(inner),
                        pass: None,
                        subpass: 0,
                    },
                ));
            };
        }
        let scratch = self.scratch.get_mut();
        scratch.reset();
        Ok(SecondaryCommandRecording {
            rec: CommandRecording {
                pool: &mut self.res,
                recording: self.recording.as_ref().unwrap(),
                graphics: Bindings::new(scratch),
                compute: Bindings::new(scratch),
                scratch,
                buffer: inner,
            },
            pass: render_pass.clone(),
            subpass,
        })
    }
    */
}

impl CommandBuffer<'_> {
    pub fn handle_mut(&mut self) -> Mut<VkCommandBuffer> {
        self.0.reborrow_mut()
    }
}
impl SecondaryCommandBuffer<'_> {
    pub fn borrow_mut(&mut self) -> Mut<VkCommandBuffer> {
        self.buf.reborrow_mut()
    }
}

impl<'a> Bindings<'a> {
    fn new(scratch: &'a bumpalo::Bump) -> Self {
        Self {
            layout: bumpalo::vec![in scratch],
            inited: bumpalo::vec![in scratch],
            pipeline: None,
        }
    }
}

impl<'a> CommandRecording<'a> {
    #[doc = crate::man_link!(vkEndCommandBuffer)]
    pub fn end(mut self) -> Result<CommandBuffer<'a>> {
        unsafe {
            (self.pool.device.fun.end_command_buffer)(
                self.buffer.handle_mut(),
            )?;
        }
        Ok(self.buffer)
    }
}
/*
impl<'a> SecondaryCommandRecording<'a> {
    #[doc = crate::man_link!(vkEndCommandBuffer)]
    pub fn end(mut self) -> Result<SecondaryCommandBuffer<'a>> {
        self.rec.end()
    }
}
*/
impl<'a> CommandRecording<'a> {
    /// Begins a render pass recorded inline. Returns [`Error::InvalidArgument`]
    /// if `framebuffer` and `render_pass` are not compatible.
    #[doc = crate::man_link!(vkCmdBeginRenderPass)]
    pub fn begin_render_pass(
        mut self, render_pass: &'a RenderPass, framebuffer: &'a Framebuffer,
        render_area: &Rect2D, clear_values: &[ClearValue],
    ) -> Result<RenderPassRecording<'a>> {
        if !framebuffer.is_compatible_with(render_pass) {
            return Err(Error::InvalidArgument);
        }
        let info = RenderPassBeginInfo {
            stype: Default::default(),
            next: Default::default(),
            render_pass: render_pass.handle(),
            framebuffer: framebuffer.handle(),
            render_area: *render_area,
            clear_values: clear_values.into(),
        };
        unsafe {
            (self.pool.device.fun.cmd_begin_render_pass)(
                self.buffer.handle_mut(),
                &info,
                SubpassContents::INLINE,
            );
        }
        Ok(RenderPassRecording { rec: self, pass: render_pass, subpass: 0 })
    }
    /*
    /// Begins a render pass recorded in secondary command buffers. Returns
    /// [`Error::InvalidArgument`] if `framebuffer` and `render_pass` are not
    /// compatible.
    #[doc = crate::man_link!(vkCmdBeginRenderPass)]
    pub fn begin_render_pass_secondary(
        mut self, render_pass: &Arc<RenderPass>,
        framebuffer: &Arc<Framebuffer>, render_area: &Rect2D,
        clear_values: &[ClearValue],
    ) -> Result<ExternalRenderPassRecording<'a>> {
        if !framebuffer.is_compatible_with(render_pass) {
            return Err(Error::InvalidArgument);
        }
        self.begin_render_pass_impl(
            render_pass,
            framebuffer,
            render_area,
            clear_values,
            SubpassContents::SECONDARY_COMMAND_BUFFERS,
        )?;
        Ok(ExternalRenderPassRecording {
            rec: self,
            pass: render_pass.clone(),
            subpass: 0,
        })
    }
    fn begin_render_pass_impl(
        &mut self, render_pass: &Arc<RenderPass>,
        framebuffer: &Arc<Framebuffer>, render_area: &Rect2D,
        clear_values: &[ClearValue], subpass_contents: SubpassContents,
    ) -> Result<()> {
        if !framebuffer.is_compatible_with(render_pass) {
            return Err(Error::InvalidArgument);
        }
        self.add_resource(render_pass.clone());
        self.add_resource(framebuffer.clone());
        let info = RenderPassBeginInfo {
            stype: Default::default(),
            next: Default::default(),
            render_pass: render_pass.handle(),
            framebuffer: framebuffer.handle(),
            render_area: *render_area,
            clear_values: clear_values.into(),
        };
        unsafe {
            (self.pool.device.fun.cmd_begin_render_pass)(
                self.buffer.mut_handle(),
                &info,
                subpass_contents,
            );
        }
        Ok(())
    }*/
}

impl<'a> RenderPassRecording<'a> {
    /// Advance to the next subpass, recorded inline. Returns
    /// [`Error::OutOfBounds`] if this is the last subpass.
    #[doc = crate::man_link!(vkCmdNextSubpass)]
    pub fn next_subpass(&mut self) -> Result<()> {
        if self.subpass >= self.pass.num_subpasses() - 1 {
            return Err(Error::OutOfBounds);
        }
        self.subpass += 1;
        unsafe {
            (self.rec.pool.device.fun.cmd_next_subpass)(
                self.rec.buffer.handle_mut(),
                SubpassContents::INLINE,
            )
        }
        Ok(())
    }
    /*
    /// Advance to the next subpass, recorded in secondary command buffers.
    /// Returns [`Error::OutOfBounds`] if this is the last subpass.
    #[doc = crate::man_link!(vkCmdNextSubpass)]
    pub fn next_subpass_secondary(
        mut self,
    ) -> Result<ExternalRenderPassRecording<'a>> {
        if self.subpass >= self.pass.num_subpasses() - 1 {
            return Err(Error::OutOfBounds);
        }
        unsafe {
            (self.rec.pool.device.fun.cmd_next_subpass)(
                self.rec.buffer.mut_handle(),
                SubpassContents::SECONDARY_COMMAND_BUFFERS,
            );
        }
        Ok(ExternalRenderPassRecording {
            rec: self.rec,
            pass: self.pass,
            subpass: self.subpass + 1,
        })
    }*/
    /// Ends the render pass. Returns [`Error::InvalidState`] if this is not the
    /// last subpass.
    #[doc = crate::man_link!(vkCmdEndRenderPass)]
    pub fn end(mut self) -> Result<CommandRecording<'a>> {
        if self.subpass != self.pass.num_subpasses() - 1 {
            return Err(Error::InvalidState);
        }
        unsafe {
            (self.rec.pool.device.fun.cmd_end_render_pass)(
                self.rec.buffer.handle_mut(),
            );
        }
        Ok(self.rec)
    }
}
/*
impl<'a> ExternalRenderPassRecording<'a> {
    /// Advance to the next subpass, recorded in secondary command buffers.
    /// Returns [`Error::OutOfBounds`] if this is the last subpass.
    #[doc = crate::man_link!(vkCmdNextSubpass)]
    pub fn next_subpass_secondary(&mut self) -> Result<()> {
        if self.subpass >= self.pass.num_subpasses() - 1 {
            return Err(Error::OutOfBounds);
        }
        self.subpass += 1;
        unsafe {
            (self.rec.pool.device.fun.cmd_next_subpass)(
                self.rec.buffer.mut_handle(),
                SubpassContents::SECONDARY_COMMAND_BUFFERS,
            )
        }
        Ok(())
    }
    /// Advance to the next subpass, recorded inline. Returns
    /// [`Error::OutOfBounds`] if this is the last subpass.
    #[doc = crate::man_link!(vkCmdNextSubpass)]
    pub fn next_subpass(mut self) -> Result<RenderPassRecording<'a>> {
        if self.subpass >= self.pass.num_subpasses() - 1 {
            return Err(Error::OutOfBounds);
        }
        unsafe {
            (self.rec.pool.device.fun.cmd_next_subpass)(
                self.rec.buffer.mut_handle(),
                SubpassContents::INLINE,
            );
        }
        Ok(RenderPassRecording {
            rec: self.rec,
            pass: self.pass,
            subpass: self.subpass + 1,
        })
    }
    /// Ends the render pass. Returns [`Error::InvalidState`] if this is not the
    /// last subpass.
    #[doc = crate::man_link!(vkCmdEndRenderPass)]
    pub fn end(mut self) -> Result<CommandRecording<'a>> {
        if self.subpass != self.pass.num_subpasses() - 1 {
            return Err(Error::InvalidState);
        }
        unsafe {
            (self.rec.pool.device.fun.cmd_end_render_pass)(
                self.rec.buffer.mut_handle(),
            );
        }
        Ok(self.rec)
    }
}
*/
#[cfg(test)]
mod test {
    use crate::vk;

    #[test]
    fn secondary_reset() -> vk::Result<()> {
        let (dev, _) = crate::test_device()?;
        let pass = vk::RenderPass::new(
            &dev,
            &vk::RenderPassCreateInfo {
                subpasses: vk::slice(&[Default::default()]),
                ..Default::default()
            },
        )?;
        let fb = vk::Framebuffer::new(
            &pass,
            Default::default(),
            vec![],
            Default::default(),
        )?;
        let mut pool1 = vk::CommandPool::new(&dev, 0)?;
        let mut pool2 = vk::CommandPool::new(&dev, 0)?;

        let sec = pool2.allocate_secondary()?;
        let mut sec = pool2.begin_secondary(sec, &pass, 0)?.end()?;
        let prim = pool1.allocate()?;
        let rec = pool1.begin(prim)?;
        let mut rec = rec.begin_render_pass_secondary(
            &pass,
            &fb,
            &Default::default(),
            Default::default(),
        )?;
        rec.execute_commands(&mut [&mut sec])?;
        let prim = rec.end()?.end()?;

        assert!(pool2.reset(Default::default()).is_err());
        assert!(pool1.reset(Default::default()).is_ok());
        assert!(pool2.reset(Default::default()).is_ok());

        assert!(pool1.free_secondary(sec).is_err());
        assert!(pool2.free(prim).is_err());

        let sec = pool1.allocate_secondary()?;
        let mut sec = pool1.begin_secondary(sec, &pass, 0)?.end()?;
        let prim = pool1.allocate()?;
        let rec = pool1.begin(prim)?;
        let mut rec = rec.begin_render_pass_secondary(
            &pass,
            &fb,
            &Default::default(),
            Default::default(),
        )?;
        rec.execute_commands(&mut [&mut sec])?;
        let _ = rec.end()?.end()?;

        assert!(pool1.reset(Default::default()).is_ok());

        Ok(())
    }

    #[test]
    fn subpass() -> vk::Result<()> {
        let (dev, _) = crate::test_device()?;
        let pass = vk::RenderPass::new(
            &dev,
            &vk::RenderPassCreateInfo {
                subpasses: vk::slice(&[Default::default(), Default::default()]),
                ..Default::default()
            },
        )?;
        let fb = vk::Framebuffer::new(
            &pass,
            Default::default(),
            vec![],
            Default::default(),
        )?;

        let mut pool = vk::CommandPool::new(&dev, 0)?;

        let buf = pool.allocate()?;
        let rec = pool.begin(buf)?;
        let rec = rec.begin_render_pass(
            &pass,
            &fb,
            &Default::default(),
            Default::default(),
        )?;
        assert!(rec.end().is_err());

        let buf = pool.allocate()?;
        let rec = pool.begin(buf)?;
        let mut rec = rec.begin_render_pass(
            &pass,
            &fb,
            &Default::default(),
            Default::default(),
        )?;
        assert!(rec.next_subpass().is_ok());
        assert!(rec.next_subpass().is_err());
        assert!(rec.next_subpass_secondary().is_err());

        pool.reset(Default::default())?;

        let buf = pool.allocate()?;
        let rec = pool.begin(buf)?;
        let mut rec = rec.begin_render_pass_secondary(
            &pass,
            &fb,
            &Default::default(),
            Default::default(),
        )?;
        assert!(rec.next_subpass_secondary().is_ok());
        assert!(rec.next_subpass_secondary().is_err());
        assert!(rec.next_subpass().is_err());

        Ok(())
    }
}
