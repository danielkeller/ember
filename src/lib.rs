#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]
#![doc = include_str!("../hello-triangle/README.md")]
#![doc = "```rust"]
#![doc = include_str!("../hello-triangle/src/main.rs")]
#![doc = "```"]

mod instance;
mod buffer;
mod cleanup_queue;
mod command_buffer;
mod descriptor_set;
mod device;
mod enums;
mod error;
mod exclusive;
mod fence;
mod ffi;
mod framebuffer;
mod image;
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
#[cfg(any(feature = "window", doc))]
#[cfg_attr(docsrs, doc(cfg(feature = "window")))]
pub mod window;
pub mod ext;

use crate::error::Result;
use crate::types::*;

macro_rules! man_link{
    ($name:ident) => {
        concat!("(see [", stringify!($name), "](https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/", stringify!($name), ".html))")
    }
}
pub(crate) use man_link;

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
    pub use crate::error::{Error, ErrorAndSelf, Result, ResultAndSelf};
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
    pub use crate::pipeline::{
        GraphicsPipelineCreateInfo, Pipeline, PipelineCache, PipelineLayout,
    };
    pub use crate::queue::Queue;
    pub use crate::queue::SubmitInfo;
    pub use crate::sampler::Sampler;
    pub use crate::types::*;
}
