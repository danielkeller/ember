// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::device::Device;
use crate::enums::*;
use crate::error::{Error, Result};
use crate::image::ImageView;
use crate::render_pass::{RenderPass, RenderPassCompat};
use crate::types::*;

/// A
#[doc = crate::spec_link!("framebuffer", "8", "_framebuffers")]
#[derive(Debug)]
pub struct Framebuffer<'a> {
    handle: Handle<VkFramebuffer>,
    render_pass_compat: RenderPassCompat,
    device: &'a Device<'a>,
}

impl<'a> Framebuffer<'a> {
    #[doc = crate::man_link!(vkCreateFrameuffer)]
    pub fn new(
        render_pass: &RenderPass<'a>, flags: FramebufferCreateFlags,
        attachments: &[&'a ImageView<'a>], size: Extent3D,
    ) -> Result<Self> {
        for iv in attachments {
            assert_eq!(iv.device(), render_pass.device());
        }
        let lim = render_pass.device().limits();
        if size.width > lim.max_framebuffer_width
            || size.height > lim.max_framebuffer_height
            || size.depth > lim.max_framebuffer_layers
        {
            return Err(Error::LimitExceeded);
        }
        let vk_attachments: Vec<_> =
            attachments.iter().map(|iv| iv.handle()).collect();
        let vk_create_info = VkFramebufferCreateInfo {
            stype: Default::default(),
            next: Default::default(),
            flags,
            render_pass: render_pass.handle(),
            attachments: (&vk_attachments).into(),
            width: size.width,
            height: size.height,
            layers: size.depth,
        };
        let mut handle = None;
        unsafe {
            (render_pass.device().fun.create_framebuffer)(
                render_pass.device().handle(),
                &vk_create_info,
                None,
                &mut handle,
            )?;
        }
        Ok(Self {
            handle: handle.unwrap(),
            render_pass_compat: render_pass.compat.clone(),
            device: render_pass.device,
        })
    }

    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkFramebuffer> {
        self.handle.borrow()
    }
    /// Returns true if this framebuffer is compatible with `pass`
    pub fn is_compatible_with(&self, pass: &RenderPass) -> bool {
        self.render_pass_compat == pass.compat
    }
}

impl Drop for Framebuffer<'_> {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun.destroy_framebuffer)(
                self.device.handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}
