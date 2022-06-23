use crate::device::Device;
use crate::error::Result;
use crate::types::*;

/// A
#[doc = crate::spec_link!("sampler", "samplers")]
#[derive(Debug, Eq)]
pub struct Sampler {
    handle: Handle<VkSampler>,
    device: Arc<Device>,
}

impl Sampler {
    #[doc = crate::man_link!(vkCreateSampler)]
    pub fn new(
        device: &Arc<Device>,
        info: &SamplerCreateInfo,
    ) -> Result<Arc<Self>> {
        let mut handle = None;
        unsafe {
            (device.fun.create_sampler)(
                device.handle(),
                info,
                None,
                &mut handle,
            )?;
        }
        Ok(Arc::new(Self { handle: handle.unwrap(), device: device.clone() }))
    }
}

impl Drop for Sampler {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun.destroy_sampler)(
                self.device.handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

impl PartialEq for Sampler {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}

impl Sampler {
    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkSampler> {
        self.handle.borrow()
    }
    /// Returns the associated device.
    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }
}
