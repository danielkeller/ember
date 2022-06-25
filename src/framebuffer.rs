use crate::enums::*;
use crate::error::{Error, Result};
use crate::image::ImageView;
use crate::render_pass::RenderPass;
use crate::types::*;

/// A
#[doc = crate::spec_link!("framebuffer", "_framebuffers")]
#[derive(Debug)]
pub struct Framebuffer {
    handle: Handle<VkFramebuffer>,
    _attachments: Vec<Arc<ImageView>>,
    render_pass: Arc<RenderPass>,
}

impl Framebuffer {
    #[doc = crate::man_link!(vkCreateFrameuffer)]
    pub fn new(
        render_pass: &Arc<RenderPass>,
        flags: FramebufferCreateFlags,
        attachments: Vec<Arc<ImageView>>,
        size: Extent3D,
    ) -> Result<Arc<Self>> {
        for iv in &attachments {
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
        Ok(Arc::new(Self {
            handle: handle.unwrap(),
            _attachments: attachments,
            render_pass: render_pass.clone(),
        }))
    }

    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkFramebuffer> {
        self.handle.borrow()
    }
    /// Returns true if this framebuffer is compatible with 'pass'
    pub fn is_compatible_with(&self, pass: &RenderPass) -> bool {
        self.render_pass.compatible(pass)
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            (self.render_pass.device.fun.destroy_framebuffer)(
                self.render_pass.device.handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}
