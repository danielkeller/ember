mod buffer;
mod cleanup_queue;
mod command_buffer;
mod descriptor_set;
mod device;
mod enums;
mod error;
mod exclusive;
pub mod ext;
mod fence;
mod ffi;
mod framebuffer;
mod image;
mod instance;
mod load;
mod memory;
mod physical_device;
mod pipeline;
mod queue;
mod render_pass;
mod sampler;
mod semaphore;
mod shader;
mod subobject;
mod types;
#[cfg(feature = "window")]
pub mod window;

use crate::error::Result;
use crate::types::*;

pub fn instance_extension_properties() -> Result<Vec<ExtensionProperties>> {
    let mut len = 0;
    let mut result = Vec::new();
    unsafe {
        let fn_ptr = load::vk_enumerate_instance_extension_properties();
        fn_ptr(None, &mut len, None)?;
        result.reserve(len as usize);
        fn_ptr(
            None,
            &mut len,
            ffi::ArrayMut::from_slice(result.spare_capacity_mut()),
        )?;
        result.set_len(len as usize);
    }
    Ok(result)
}

pub mod vk {
    pub use crate::buffer::{Buffer, BufferWithoutMemory};
    pub use crate::command_buffer::command::{
        BufferMemoryBarrier, ImageMemoryBarrier,
    };
    pub use crate::command_buffer::{
        CommandBuffer, CommandPool, CommandRecording, RenderPassRecording,
        SecondaryCommandBuffer,
    };
    pub use crate::descriptor_set::{
        DescriptorBufferInfo, DescriptorSetLayout, DescriptorSetLayoutBinding,
        DescriptorSetUpdateBuilder,
    };
    pub use crate::device::Device;
    pub use crate::enums::*;
    pub use crate::ext;
    pub use crate::ext::khr_swapchain::{
        CreateSwapchainFrom, KHRSwapchain, SwapchainCreateInfoKHR,
    };
    pub use crate::fence::{Fence, PendingFence};
    pub use crate::ffi::*;
    pub use crate::image::{
        Image, ImageView, ImageViewCreateInfo, ImageWithoutMemory,
    };
    pub use crate::instance::Instance;
    pub use crate::instance_extension_properties;
    pub use crate::memory::DeviceMemory;
    pub use crate::physical_device::PhysicalDevice;
    pub use crate::pipeline::{GraphicsPipelineCreateInfo, Pipeline};
    pub use crate::queue::Queue;
    pub use crate::queue::SubmitInfo;
    pub use crate::sampler::Sampler;
    pub use crate::types::*;
}
