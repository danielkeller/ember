#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]
#![doc = include_str!("../INDEX.md")]
#![doc = include_str!("../hello-triangle/README.md")]
#![doc = "```rust"]
#![doc = include_str!("../hello-triangle/src/main.rs")]
#![doc = "```"]

mod instance;
mod physical_device;
mod device;
mod queue;
mod buffer;
mod image;
mod memory;
mod cleanup_queue;
mod command_buffer;
mod descriptor_set;
mod enums;
mod error;
mod exclusive;
mod fence;
mod semaphore;
mod ffi;
mod framebuffer;
mod load;
mod shader;
mod pipeline;
mod render_pass;
mod sampler;
mod subobject;
mod types;
#[cfg(any(feature = "window", doc))]
#[cfg_attr(docsrs, doc(cfg(feature = "window")))]
pub mod window;
pub mod ext;
#[cfg(doc)]
pub mod macos_instructions;

use crate::error::Result;
use crate::types::*;

macro_rules! man_link{
    ($name:ident) => {
        concat!("(see [", stringify!($name), "](https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/", stringify!($name), ".html))")
    }
}
pub(crate) use man_link;
macro_rules! spec_link{
    ($text:literal, $hash:literal) => {
        concat!("[", $text, "](https://www.khronos.org/registry/vulkan/specs/1.3-extensions/html/vkspec.html#", $hash, ")")
    }
}
pub(crate) use spec_link;

#[doc = crate::man_link!(vkEnumerateInstanceExtensionProperties)]
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
        CommandBuffer, CommandPool, CommandRecording,
        ExternalRenderPassRecording, RenderPassRecording,
        SecondaryCommandBuffer, SecondaryCommandRecording,
    };
    pub use crate::descriptor_set::{
        update::DescriptorBufferInfo, update::DescriptorSetUpdate,
        update::DescriptorSetUpdateBuilder, update::DescriptorSetUpdates,
        DescriptorPool, DescriptorSet, DescriptorSetLayout,
        DescriptorSetLayoutBinding,
    };
    pub use crate::device::Device;
    pub use crate::enums::*;
    pub use crate::error::{Error, ErrorAndSelf, Result, ResultAndSelf};
    pub use crate::ext;
    pub use crate::ext::khr_swapchain::{
        CreateSwapchainFrom, SwapchainCreateInfoKHR,
    };
    pub use crate::fence::{Fence, PendingFence};
    pub use crate::ffi::*;
    pub use crate::framebuffer::Framebuffer;
    pub use crate::image::{
        Image, ImageView, ImageViewCreateInfo, ImageWithoutMemory,
    };
    pub use crate::instance::Instance;
    pub use crate::instance_extension_properties;
    pub use crate::memory::{DeviceMemory, MappedMemory};
    pub use crate::physical_device::PhysicalDevice;
    pub use crate::pipeline::{
        GraphicsPipelineCreateInfo, Pipeline, PipelineCache, PipelineLayout,
    };
    pub use crate::queue::Queue;
    pub use crate::queue::SubmitInfo;
    pub use crate::render_pass::RenderPass;
    pub use crate::sampler::Sampler;
    pub use crate::semaphore::Semaphore;
    pub use crate::shader::ShaderModule;
    pub use crate::types::*;
}
