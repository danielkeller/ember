// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]
#![doc = include_str!("../INDEX.md")]
#![doc = include_str!("../hello-triangle/README.md")]
#![doc = "```ignore"]
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

use crate::types::*;

macro_rules! man_link{
    ($name:ident) => {
        concat!("(see [`", stringify!($name), "`](https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/", stringify!($name), ".html))")
    }
}

pub(crate) use man_link;
macro_rules! spec_link {
    ($text:literal, $chapter:literal, $hash:literal) => {
        concat!(
            "[",
            $text,
            "](https://renderdoc.org/vkspec_chunked/chap",
            $chapter,
            ".html#",
            $hash,
            ")"
        )
    };
}
pub(crate) use spec_link;

use core::marker::PhantomData as PD;

struct Pool;
struct RecSession<'pool>(&'pool mut Pool);

impl Pool {
    fn reset<'pool>(&'pool mut self) -> RecSession<'pool> {
        RecSession(self)
    }
}
impl<'pool> RecSession<'pool> {
    fn begin<'rec>(&'rec mut self) -> CmdRecording<'rec, 'pool> {
        CmdRecording(self)
    }
    fn record(&mut self) -> CmdBuf<'pool> {
        CmdBuf(PD)
    }
}

struct CmdBuf<'a>(PD<&'a ()>);
struct CmdRecording<'rec, 'pool: 'rec>(&'rec mut RecSession<'pool>);

impl<'rec, 'pool> CmdRecording<'rec, 'pool> {
    fn record_thing(&mut self, _: &'pool Object) {}
    fn record_secondary(&mut self, _: &'pool mut CmdBuf<'pool>) {}
    fn end(self) -> CmdBuf<'pool> {
        CmdBuf(PD)
    }
}
impl CmdBuf<'_> {
    fn submit(&mut self) {}
}

struct Object;

struct PoolSet;

impl PoolSet {
    fn get_pool<'pool>(&'pool self) -> RecSession<'pool> {
        RecSession(todo!())
    }
    fn reset(&mut self) {}
}

fn device<'a>() -> &'a vk::Device {
    todo!()
}

fn foo() {
    let mut pool = Pool;
    let mut pool1 = Pool;
    let mut pool1_contents = pool1.reset();
    let mut pool_contents = pool.reset();

    let mut b = None;
    std::thread::scope(|s| {
        s.spawn(|| {
            let mut b_rec = pool_contents.begin();
            b_rec.record_thing(&Object);
            b = Some(b_rec.end());
        });
    });

    let foo = 5;
    std::thread::scope(|s| {
        s.spawn(|| {
            foo == 7;
            drop(foo);
        });
    });

    let mut b1_rec = pool1_contents.begin();
    b1_rec.record_secondary(b.as_mut().unwrap());
    let mut b1 = b1_rec.end();
    drop(pool1_contents);
    b1.submit();
    pool.reset();

    let mut poolset = PoolSet;
    // let mut cmds = vec![];
    // [&Object, &Object]
    //     .into_par_iter()
    //     .map_init(
    //         || poolset.get_pool(),
    //         |pool, i| {
    //             let mut rec = pool.begin();
    //             rec.record_thing(i);
    //             rec.end()
    //         },
    //     )
    //     .collect_into_vec(&mut cmds);
    // cmds[0].submit();
    poolset.reset();
}

struct Scope<'s>(std::marker::PhantomData<&'s mut &'s ()>);

impl<'s> Scope<'s> {
    fn run_mut(&mut self, _: &'s mut ()) {}
    fn run(&mut self, _: &'s ()) {}
}

fn loop_scope<F, T>(vs: &mut [T], mut f: F)
where
    F: for<'a> FnMut(&mut Scope<'a>, &'a mut T),
{
    loop {
        for v in vs.iter_mut() {
            let mut s = Scope(std::marker::PhantomData);
            f(&mut s, v)
        }
    }
}

fn test_the_scope() {
    let mut vs = [(), ()];
    let v1 = ();
    loop_scope(&mut vs, |s, v| {
        s.run_mut(v);
    });
}

#[doc = crate::man_link!(vkEnumerateInstanceExtensionProperties)]
pub fn instance_extension_properties() -> Vec<ExtensionProperties> {
    let mut len = 0;
    let mut result = Vec::new();
    unsafe {
        let fn_ptr = load::vk_enumerate_instance_extension_properties();
        fn_ptr(None, &mut len, None).unwrap();
        result.reserve(len as usize);
        fn_ptr(
            None,
            &mut len,
            ffi::ArrayMut::from_slice(result.spare_capacity_mut()),
        )
        .unwrap();
        result.set_len(len as usize);
    }
    result
}

#[cfg(test_disabled)]
pub(crate) fn test_device() -> Result<(Arc<device::Device>, queue::Queue)> {
    let inst = vk::Instance::new(&Default::default())?;
    let (dev, mut qs) = vk::Device::new(
        &inst.enumerate_physical_devices()?[0],
        &vk::DeviceCreateInfo {
            queue_create_infos: vk::slice(&[vk::DeviceQueueCreateInfo {
                queue_priorities: vk::slice(&[1.0]),
                ..Default::default()
            }]),
            enabled_features: Some(&vk::PhysicalDeviceFeatures {
                robust_buffer_access: vk::True,
                ..Default::default()
            }),
            ..Default::default()
        },
    )?;
    Ok((dev, qs.remove(0).remove(0)))
}

/// Vulkan core functionality.
///
/// This module is intended to be imported qualified; ie `use maia::vk;`
pub mod vk {
    pub use crate::buffer::{Buffer, BufferWithoutMemory};
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
    pub use crate::enums::Bool::{False, True};
    pub use crate::enums::*;
    pub use crate::error::{OutOfDeviceMemory, OutOfPoolMemory, VkResult};
    pub use crate::ext;
    pub use crate::ext::khr_swapchain::SwapchainCreateInfoKHR;
    pub use crate::fence::{Fence, PendingFence};
    pub use crate::ffi::*;
    pub use crate::image::{
        Framebuffer, Image, ImageView, ImageViewCreateInfo, ImageWithoutMemory,
    };
    pub use crate::instance::Instance;
    pub use crate::instance_extension_properties;
    pub use crate::memory::{DeviceMemory, MappedMemory};
    pub use crate::physical_device::PhysicalDevice;
    pub use crate::pipeline::{
        GraphicsPipelineCreateInfo, Pipeline, PipelineLayout,
    };
    pub use crate::queue::Queue;
    pub use crate::queue::SubmitScope;
    pub use crate::render_pass::RenderPass;
    pub use crate::sampler::Sampler;
    pub use crate::semaphore::Semaphore;
    pub use crate::shader::ShaderModule;
    pub use crate::types::*;
}
