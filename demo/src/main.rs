// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::cell::Cell;
use std::future::{poll_fn, Future};
use std::io::Write;
use std::mem::size_of;
use std::ops::Deref;
use std::pin::{pin, Pin};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Instant;
use std::{collections::HashMap, ops::DerefMut};

use anyhow::Context as _;
use maia::vk;
use scoped_tls::scoped_thread_local;
use ultraviolet::{Mat4, Vec3};
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

fn find_right_directory() -> anyhow::Result<()> {
    let exe = std::env::current_exe()?;
    let exedir = exe.parent().unwrap();
    if let Some(dirname) = exedir.file_name() {
        if dirname == std::ffi::OsStr::new("debug")
            || dirname == std::ffi::OsStr::new("release")
        {
            // Running in workspace
            if let Some(workspace) = exedir.parent().and_then(|d| d.parent()) {
                std::env::set_current_dir(workspace.join("demo"))?;
            }
        } else if dirname == std::ffi::OsStr::new("MacOS") {
            if let Some(appdir) = exe.parent().unwrap().parent() {
                // App bundle
                std::env::set_current_dir(appdir.join("Resources"))?;
            }
        } else {
            std::env::set_current_dir(exedir)?;
        }
    }
    Ok(())
}

#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
struct Vertex {
    pos: [f32; 4],
    uv: [f32; 2],
}

const VERTEX_DATA: [Vertex; 4] = [
    Vertex { pos: [-0.7, -0.7, 0.0, 1.0], uv: [1.0, 1.0] },
    Vertex { pos: [-0.7, 0.7, 0.0, 1.0], uv: [1.0, 0.0] },
    Vertex { pos: [0.7, -0.7, 0.0, 1.0], uv: [0.0, 1.0] },
    Vertex { pos: [0.7, 0.7, 0.0, 1.0], uv: [0.0, 0.0] },
];
const INDEX_DATA: [u16; 6] = [0, 1, 2, 2, 1, 3];

#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
struct MVP {
    model: Mat4,
    view: Mat4,
    proj: Mat4,
}

fn required_instance_extensions() -> anyhow::Result<Vec<vk::Str<'static>>> {
    let mut result = vec![];
    for ext in vk::instance_extension_properties()? {
        if ext.extension_name == vk::ext::GET_PHYSICAL_DEVICE_PROPERTIES2 {
            result.push(vk::ext::GET_PHYSICAL_DEVICE_PROPERTIES2)
        } else if ext.extension_name == vk::ext::PORTABILITY_ENUMERATION {
            result.push(vk::ext::PORTABILITY_ENUMERATION);
        }
    }
    Ok(result)
}

fn pick_physical_device(phys: &[vk::PhysicalDevice]) -> vk::PhysicalDevice {
    let discr = vk::PhysicalDeviceType::DISCRETE_GPU;
    let int = vk::PhysicalDeviceType::INTEGRATED_GPU;
    phys.iter()
        .find(|p| p.properties().device_type == discr)
        .or_else(|| phys.iter().find(|p| p.properties().device_type == int))
        .unwrap_or(&phys[0])
        .clone()
}

fn pick_queue_family(
    phy: &vk::PhysicalDevice, surf: &vk::ext::SurfaceKHR,
    window: &winit::window::Window,
) -> anyhow::Result<u32> {
    for (num, props) in phy.queue_family_properties().iter().enumerate() {
        if !(props.queue_flags & vk::QueueFlags::GRAPHICS).is_empty()
            && surf.support(phy, num as u32)?
            && maia::window::presentation_support(phy, num as u32, window)
        {
            return Ok(num as u32);
        }
    }
    anyhow::bail!("No graphics queue")
}

fn required_device_extensions(
    phy: &vk::PhysicalDevice,
) -> anyhow::Result<&'static [vk::Str<'static>]> {
    let exts = phy.device_extension_properties()?;
    if exts.iter().any(|e| e.extension_name == vk::ext::PORTABILITY_SUBSET) {
        Ok(&[vk::ext::PORTABILITY_SUBSET, vk::ext::SWAPCHAIN])
    } else {
        Ok(&[vk::ext::SWAPCHAIN])
    }
}

fn memory_type(
    phy: &vk::PhysicalDevice, desired: vk::MemoryPropertyFlags,
) -> u32 {
    let mem_props = phy.memory_properties();
    for (num, props) in mem_props.memory_types.iter().enumerate() {
        if props.property_flags & desired == desired {
            return num as u32;
        }
    }
    panic!("Desired memory type not found")
}

fn upload_data(
    device: &vk::Device, queue: &mut vk::Queue, cmd_pool: &vk::CommandPool,
    src: &[u8], dst: &vk::Buffer, dst_stage_mask: vk::PipelineStageFlags,
    dst_access_mask: vk::AccessFlags,
) -> anyhow::Result<()> {
    let staging_buffer = vk::BufferWithoutMemory::new(
        device,
        &vk::BufferCreateInfo {
            size: src.len() as u64,
            usage: vk::BufferUsageFlags::TRANSFER_SRC,
            ..Default::default()
        },
    )?;
    let mem_size = staging_buffer.memory_requirements().size;
    let host_mem = memory_type(
        device.physical_device(),
        vk::MemoryPropertyFlags::HOST_VISIBLE
            | vk::MemoryPropertyFlags::HOST_COHERENT,
    );
    let memory = vk::DeviceMemory::new(device, mem_size, host_mem)?;
    let mut memory = memory.map(0, src.len())?;
    let staging_buffer = vk::Buffer::new(staging_buffer, &memory, 0)?;
    memory.as_slice_mut().copy_from_slice(src);

    let mut transfer = cmd_pool.begin();
    transfer.copy_buffer(
        &staging_buffer,
        dst,
        &[vk::BufferCopy { size: src.len() as u64, ..Default::default() }],
    )?;
    transfer.memory_barrier(
        vk::PipelineStageFlags::TRANSFER,
        dst_stage_mask,
        vk::AccessFlags::TRANSFER_WRITE,
        dst_access_mask,
    );
    let transfer = transfer.end()?;
    queue.submit(|s| s.command(transfer));
    Ok(())
}

fn upload_image(
    device: &vk::Device, queue: &mut vk::Queue, image: &vk::Image,
    cmd_pool: &vk::CommandPool,
) -> anyhow::Result<()> {
    let image_file = std::fs::File::open("assets/texture.jpg")?;
    let mut image_data =
        jpeg_decoder::Decoder::new(std::io::BufReader::new(image_file));
    let image_data = image_data.decode()?;

    let staging_buffer = vk::BufferWithoutMemory::new(
        &device,
        &vk::BufferCreateInfo {
            size: (image_data.len() / 3 * 4) as u64,
            usage: vk::BufferUsageFlags::TRANSFER_SRC,
            ..Default::default()
        },
    )?;
    let mem_size = staging_buffer.memory_requirements().size;
    let host_mem = memory_type(
        device.physical_device(),
        vk::MemoryPropertyFlags::HOST_VISIBLE
            | vk::MemoryPropertyFlags::HOST_COHERENT,
    );
    let mut memory = vk::DeviceMemory::new(&device, mem_size, host_mem)?;
    let mut memory = memory.map(0, mem_size as usize)?;
    let staging_buffer = vk::Buffer::new(staging_buffer, &memory, 0)?;
    for (src, dst) in (image_data.chunks_exact(3))
        .zip(memory.as_slice_mut().chunks_exact_mut(4))
    {
        dst[0..3].clone_from_slice(src);
        dst[4] = 255;
    }
    memory.unmap();

    let mut transfer = cmd_pool.begin();
    transfer.image_barrier(
        image,
        vk::PipelineStageFlags::TOP_OF_PIPE,
        vk::PipelineStageFlags::TRANSFER,
        vk::AccessFlags::default(),
        vk::AccessFlags::TRANSFER_WRITE,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
    );
    transfer.copy_buffer_to_image(
        &staging_buffer,
        image,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        &[vk::BufferImageCopy {
            image_extent: vk::Extent3D { width: 512, height: 512, depth: 1 },
            ..Default::default()
        }],
    )?;
    transfer.image_barrier(
        image,
        vk::PipelineStageFlags::TRANSFER,
        vk::PipelineStageFlags::TOP_OF_PIPE,
        vk::AccessFlags::TRANSFER_WRITE,
        vk::AccessFlags::MEMORY_READ,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    );
    let transfer = transfer.end()?;
    queue.submit(|s| s.command(transfer));
    Ok(())
}

macro_rules! shader {
    ($name:literal) => {
        &{
            const BYTES: &[u8] =
                include_bytes!(concat!(env!("OUT_DIR"), "/shaders/", $name));
            let mut words = vec![0u32; BYTES.len() / 4];
            bytemuck::cast_slice_mut(&mut words).copy_from_slice(BYTES);
            words
        }
    };
}

async fn main_loop() -> anyhow::Result<()> {
    find_right_directory().context("Changing dirs")?;
    let window = EVENT_LOOP
        .with(|evtloop| evtloop.create_window(Window::default_attributes()))?;
    window.set_title("Maia Demo");

    let mut instance_exts = vec![];
    instance_exts
        .extend(maia::window::required_instance_extensions(&window)?.iter());
    instance_exts.extend(required_instance_extensions()?.iter());
    let inst = vk::Instance::new(&vk::InstanceCreateInfo {
        enabled_extension_names: instance_exts.as_slice().into(),
        flags: if instance_exts.contains(&vk::ext::PORTABILITY_ENUMERATION) {
            vk::InstanceCreateFlags::INSTANCE_CREATE_ENUMERATE_PORTABILITY_BIT_KHR
        } else {
            Default::default()
        },
        ..Default::default()
    })?;

    let surf = maia::window::create_surface(&inst, &window, &window)?;

    let phy = pick_physical_device(&inst.enumerate_physical_devices()?);
    let queue_family = pick_queue_family(&phy, &surf, &window)?;
    if !surf.surface_formats(&phy)?.iter().any(|f| {
        f == &vk::SurfaceFormatKHR {
            format: vk::Format::B8G8R8A8_UNORM,
            color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR_KHR,
        }
    }) {
        anyhow::bail!("Desired surface format not found");
    }

    let device_extensions = required_device_extensions(&phy)?;
    let (device, mut queues) = vk::Device::new(
        &phy,
        &vk::DeviceCreateInfo {
            queue_create_infos: vk::slice(&[vk::DeviceQueueCreateInfo {
                queue_family_index: queue_family,
                queue_priorities: vk::slice(&[1.0]),
                ..Default::default()
            }]),
            enabled_extension_names: device_extensions.into(),
            enabled_features: Some(&vk::PhysicalDeviceFeatures {
                // Allow the use of in-RAM vertex buffers
                robust_buffer_access: vk::True,
                ..Default::default()
            }),
            ..Default::default()
        },
    )?;
    let mut queue = queues.remove(0).remove(0);

    let mut acquire_sem = vk::Semaphore::new(&device)?;

    let window_size = window.inner_size();
    let mut swapchain_size =
        vk::Extent2D { width: window_size.width, height: window_size.height };
    let mut swapchain = vk::ext::SwapchainKHR::new(
        &device,
        surf,
        &vk::SwapchainCreateInfoKHR {
            min_image_count: 3,
            image_format: vk::Format::B8G8R8A8_SRGB,
            image_extent: swapchain_size,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT
                | vk::ImageUsageFlags::TRANSFER_DST,
            ..Default::default()
        },
    )?;
    let mut swapchain_images = swapchain.images();
    let mut present_semaphores = [
        vk::Semaphore::new(&device)?,
        vk::Semaphore::new(&device)?,
        vk::Semaphore::new(&device)?,
    ];

    let mut cmd_pool = vk::CommandPool::new(&device, queue_family)?;

    let vertex_size = std::mem::size_of_val(&VERTEX_DATA);
    let index_size = std::mem::size_of_val(&INDEX_DATA);

    let device_mem = memory_type(
        device.physical_device(),
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    );

    let vertex_buffer = vk::BufferWithoutMemory::new(
        &device,
        &vk::BufferCreateInfo {
            size: vertex_size as u64,
            usage: vk::BufferUsageFlags::VERTEX_BUFFER
                | vk::BufferUsageFlags::TRANSFER_DST,
            ..Default::default()
        },
    )?;
    let index_buffer = vk::BufferWithoutMemory::new(
        &device,
        &vk::BufferCreateInfo {
            size: index_size as u64,
            usage: vk::BufferUsageFlags::INDEX_BUFFER
                | vk::BufferUsageFlags::TRANSFER_DST,
            ..Default::default()
        },
    )?;
    let vert_req = vertex_buffer.memory_requirements();
    let ind_req = index_buffer.memory_requirements();
    assert_ne!(vert_req.memory_type_bits & (1 << device_mem), 0);
    assert_ne!(ind_req.memory_type_bits & (1 << device_mem), 0);
    let ind_start =
        (vert_req.size + ind_req.alignment - 1) & !(ind_req.alignment - 1);
    let memory =
        &vk::DeviceMemory::new(&device, ind_start + ind_req.size, device_mem)?;

    let vertex_buffer = vk::Buffer::new(vertex_buffer, memory, 0)?;
    let index_buffer = vk::Buffer::new(index_buffer, memory, ind_start)?;

    upload_data(
        &device,
        &mut queue,
        &mut cmd_pool,
        bytemuck::bytes_of(&VERTEX_DATA),
        &vertex_buffer,
        vk::PipelineStageFlags::VERTEX_INPUT,
        vk::AccessFlags::VERTEX_ATTRIBUTE_READ,
    )?;

    upload_data(
        &device,
        &mut queue,
        &mut cmd_pool,
        bytemuck::bytes_of(&INDEX_DATA),
        &index_buffer,
        vk::PipelineStageFlags::VERTEX_INPUT,
        vk::AccessFlags::INDEX_READ,
    )?;
    let uniform_buffer = vk::BufferWithoutMemory::new(
        &device,
        &vk::BufferCreateInfo {
            size: size_of::<MVP>() as u64,
            usage: vk::BufferUsageFlags::UNIFORM_BUFFER,
            ..Default::default()
        },
    )?;
    let mut uniform_memory = vk::DeviceMemory::new(
        &device,
        uniform_buffer.memory_requirements().size,
        memory_type(
            device.physical_device(),
            vk::MemoryPropertyFlags::DEVICE_LOCAL
                | vk::MemoryPropertyFlags::HOST_VISIBLE
                | vk::MemoryPropertyFlags::HOST_COHERENT,
        ),
    )?;
    let mut mapped = uniform_memory.map(0, size_of::<MVP>())?;
    let uniform_buffer = vk::Buffer::new(uniform_buffer, &mapped, 0)?;

    let image = vk::ImageWithoutMemory::new(
        &device,
        &vk::ImageCreateInfo {
            format: vk::Format::R8G8B8A8_SRGB,
            extent: vk::Extent3D { width: 512, height: 512, depth: 1 },
            usage: vk::ImageUsageFlags::TRANSFER_DST
                | vk::ImageUsageFlags::SAMPLED,
            ..Default::default()
        },
    )?;
    let image_reqs = image.memory_requirements();
    assert_ne!(image_reqs.memory_type_bits & (1 << device_mem), 0);
    let image_memory =
        vk::DeviceMemory::new(&device, image_reqs.size, device_mem)?;
    let image = vk::Image::new(image, &image_memory, 0)?;
    upload_image(&device, &mut queue, &image, &mut cmd_pool)?;
    let image_view = vk::ImageView::new(
        &image,
        &vk::ImageViewCreateInfo {
            format: vk::Format::R8G8B8A8_SRGB,
            ..Default::default()
        },
    )?;

    let render_pass = vk::RenderPass::new(
        &device,
        &vk::RenderPassCreateInfo {
            attachments: vk::slice(&[vk::AttachmentDescription {
                format: vk::Format::B8G8R8A8_SRGB,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::STORE,
                initial_layout: vk::ImageLayout::UNDEFINED,
                final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
                ..Default::default()
            }]),
            subpasses: vk::slice(&[vk::SubpassDescription {
                color_attachments: &[vk::AttachmentReference {
                    attachment: 0,
                    layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                }],
                ..Default::default()
            }
            .try_into()?]),
            ..Default::default()
        },
    )?;

    let mut swapchain_imageviews = vec![];
    for img in swapchain_images.images() {
        swapchain_imageviews.push(vk::ImageView::new(
            img,
            &vk::ImageViewCreateInfo {
                format: vk::Format::B8G8R8A8_SRGB,
                ..Default::default()
            },
        )?);
    }
    let mut framebuffers = vec![];
    for img_view in &swapchain_imageviews {
        framebuffers.push((|swapchain_size, img_view| {
            vk::Framebuffer::new(
                &render_pass,
                Default::default(),
                &[img_view],
                swapchain_size,
            )
        })(swapchain_size.into(), img_view)?);
    }

    let vertex_shader =
        vk::ShaderModule::new(&device, shader!("triangle.vert"))?;
    let fragment_shader =
        vk::ShaderModule::new(&device, shader!("triangle.frag"))?;

    let sampler = vk::Sampler::new(&device, &Default::default())?;
    let descriptor_set_layout = vk::DescriptorSetLayout::new(
        &device,
        vec![
            vk::DescriptorSetLayoutBinding {
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::VERTEX,
                immutable_samplers: vec![],
            },
            vk::DescriptorSetLayoutBinding {
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
                immutable_samplers: vec![sampler],
            },
        ],
    )?;
    let mut descriptor_pool = vk::DescriptorPool::new(
        &device,
        1,
        &[
            vk::DescriptorPoolSize {
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
            },
            vk::DescriptorPoolSize {
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 1,
            },
        ],
    )?;
    let mut desc_set =
        vk::DescriptorSet::new(&mut descriptor_pool, &descriptor_set_layout)?;

    let mut update = vk::DescriptorSetUpdateBuilder::new(&device);
    update
        .begin()
        .dst_set(&mut desc_set)
        .uniform_buffers(
            0,
            0,
            &[vk::DescriptorBufferInfo {
                buffer: &uniform_buffer,
                offset: 0,
                range: None,
            }],
        )?
        .combined_image_samplers(
            1,
            0,
            &[(&image_view, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)],
        )?
        .end();

    let pipeline_layout = vk::PipelineLayout::new(
        &device,
        Default::default(),
        &[descriptor_set_layout],
        vec![],
    )?;

    let pipeline =
        vk::Pipeline::new_graphics(vk::GraphicsPipelineCreateInfo {
            stages: &[
                vk::PipelineShaderStageCreateInfo::vertex(&vertex_shader),
                vk::PipelineShaderStageCreateInfo::fragment(&fragment_shader),
            ],
            vertex_input_state: &vk::PipelineVertexInputStateCreateInfo {
                vertex_binding_descriptions: vk::slice(&[
                    vk::VertexInputBindingDescription {
                        binding: 0,
                        stride: size_of::<Vertex>() as u32,
                        input_rate: vk::VertexInputRate::VERTEX,
                    },
                ]),
                vertex_attribute_descriptions: vk::slice(&[
                    vk::VertexInputAttributeDescription {
                        location: 0,
                        binding: 0,
                        format: vk::Format::R32G32B32A32_SFLOAT,
                        offset: 0,
                    },
                    vk::VertexInputAttributeDescription {
                        location: 1,
                        binding: 0,
                        format: vk::Format::R32G32_SFLOAT,
                        offset: 16,
                    },
                ]),
                ..Default::default()
            },
            input_assembly_state: &vk::PipelineInputAssemblyStateCreateInfo {
                topology: vk::PrimitiveTopology::TRIANGLE_LIST,
                ..Default::default()
            },
            tessellation_state: None,
            viewport_state: &vk::PipelineViewportStateCreateInfo {
                viewports: vk::slice(&[Default::default()]),
                scissors: vk::slice(&[vk::Rect2D {
                    offset: Default::default(),
                    extent: vk::Extent2D { width: 3840, height: 2160 },
                }]),
                ..Default::default()
            },
            rasterization_state: &Default::default(),
            multisample_state: &Default::default(),
            depth_stencil_state: None,
            color_blend_state: &Default::default(),
            dynamic_state: Some(&vk::PipelineDynamicStateCreateInfo {
                dynamic_states: vk::slice(&[vk::DynamicState::VIEWPORT]),
                ..Default::default()
            }),
            layout: Arc::new(pipeline_layout),
            render_pass,
            subpass: 0,
        })?;

    let begin = Instant::now();

    loop {
        window.request_redraw();
        use WindowEvent::*;
        let evt =
            winit_event(|evt| matches!(evt, RedrawRequested | CloseRequested));
        if let CloseRequested = evt.await {
            return Ok(());
        }

        let draw_size = window.inner_size();
        let draw_size =
            vk::Extent2D { width: draw_size.width, height: draw_size.height };

        if draw_size != swapchain_size {
            swapchain_size = draw_size;
            drop(framebuffers);
            drop(swapchain_imageviews);
            swapchain.recreate(&vk::SwapchainCreateInfoKHR {
                min_image_count: 3,
                image_format: vk::Format::B8G8R8A8_SRGB,
                image_extent: swapchain_size,
                image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT
                    | vk::ImageUsageFlags::TRANSFER_DST,
                ..Default::default()
            })?;
            swapchain_images = swapchain.images();
            swapchain_imageviews = vec![];
            for img in swapchain_images.images() {
                swapchain_imageviews.push(vk::ImageView::new(
                    img,
                    &vk::ImageViewCreateInfo {
                        format: vk::Format::B8G8R8A8_SRGB,
                        ..Default::default()
                    },
                )?);
            }
            framebuffers = vec![];
            for img_view in &swapchain_imageviews {
                framebuffers.push((|swapchain_size, img_view| {
                    vk::Framebuffer::new(
                        &render_pass,
                        Default::default(),
                        &[img_view],
                        swapchain_size,
                    )
                })(
                    swapchain_size.into(), img_view
                )?);
            }
        }

        let (img, _subopt) =
            swapchain_images.acquire_next_image(&mut acquire_sem, u64::MAX)?;
        let framebuffer = &framebuffers[img];
        let present_sem = &mut present_semaphores[img];

        let time = Instant::now().duration_since(begin);

        let mvp: &mut MVP = bytemuck::from_bytes_mut(mapped.as_slice_mut());
        mvp.model = Mat4::from_rotation_y(time.as_secs_f32() * 2.0);
        mvp.view = Mat4::look_at(
            Vec3::new(1., 1., 1.),
            Vec3::zero(),
            Vec3::new(0., 1., 0.),
        );
        mvp.proj = ultraviolet::projection::perspective_infinite_z_vk(
            std::f32::consts::FRAC_PI_2,
            draw_size.width as f32 / draw_size.height as f32,
            0.1,
        );

        cmd_pool.reset()?;
        // let subpass = cmd_pool.allocate_secondary()?;
        // let mut subpass = cmd_pool.begin_secondary(subpass, &render_pass, 0)?;
        // subpass.set_viewport(&vk::Viewport {
        //     x: 0.0,
        //     y: 0.0,
        //     width: draw_size.width as f32,
        //     height: draw_size.height as f32,
        //     min_depth: 0.0,
        //     max_depth: 1.0,
        // });
        // subpass.bind_pipeline(&pipeline);
        // subpass.bind_vertex_buffers(0, &[(&vertex_buffer, 0)])?;
        // subpass.bind_index_buffer(&index_buffer, 0, vk::IndexType::UINT16)?;
        // subpass.bind_descriptor_sets(
        //     vk::PipelineBindPoint::GRAPHICS,
        //     &pipeline_layout,
        //     0,
        //     &[&desc_set],
        //     &[],
        // )?;
        // subpass.draw_indexed(6, 1, 0, 0, 0)?;
        // let mut subpass = subpass.end()?;

        let mut pass = cmd_pool.begin().begin_render_pass(
            &render_pass,
            &framebuffer,
            &vk::Rect2D {
                offset: Default::default(),
                extent: vk::Extent2D {
                    width: draw_size.width,
                    height: draw_size.height,
                },
            },
            &[vk::ClearValue {
                color: vk::ClearColorValue { f32: [0.1, 0.2, 0.3, 1.0] },
            }],
        )?;
        pass.set_viewport(&vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: draw_size.width as f32,
            height: draw_size.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        });
        pass.bind_pipeline(&pipeline);
        pass.bind_vertex_buffers(0, [&vertex_buffer], [0])?;
        pass.bind_index_buffer(&index_buffer, 0, vk::IndexType::UINT16)?;
        pass.bind_descriptor_sets(
            vk::PipelineBindPoint::GRAPHICS,
            &pipeline_layout,
            0,
            &[&desc_set],
            &[],
        )?;
        pass.draw_indexed(6, 1, 0, 0, 0)?;
        let mut buf = pass.end()?.end()?;

        let present_signal = queue.submit(|s| {
            s.acquire_next_image(
                &mut swapchain_images,
                0,
                vk::PipelineStageFlags::TOP_OF_PIPE,
            );
            // s.wait(acquire_sem, vk::PipelineStageFlags::TOP_OF_PIPE);
            s.command(buf);
            s.signal(&mut present_sem)
        });
        swapchain_images.present(&mut queue, img, present_sem)?;
    }
}

struct Waker;
impl std::task::Wake for Waker {
    fn wake(self: Arc<Self>) {}
}

struct App<'a, F> {
    fut: Pin<&'a mut F>,
}

scoped_thread_local!(static EVENT_LOOP: ActiveEventLoop);
thread_local!(static WINDOW_EVENT: Cell<Option<WindowEvent>> = Cell::new(None));

async fn winit_event(f: impl Fn(&WindowEvent) -> bool) -> WindowEvent {
    poll_fn(|cx| {
        if !EVENT_LOOP.is_set() {
            panic!("winit_event() called outside of event loop")
        }
        if let Some(val) = WINDOW_EVENT.take() {
            if f(&val) {
                return Poll::Ready(val);
            } else {
                WINDOW_EVENT.replace(Some(val));
            }
        }
        cx.waker().wake_by_ref();
        Poll::Pending
    })
    .await
}

impl<'a, F> winit::application::ApplicationHandler<()> for App<'a, F>
where
    F: Future<Output = ()>,
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let waker = Arc::new(Waker).into();
        EVENT_LOOP.set(event_loop, || {
            if let Poll::Ready(()) =
                self.fut.as_mut().poll(&mut Context::from_waker(&waker))
            {
                event_loop.exit();
            }
        });
    }

    fn window_event(
        &mut self, event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId, event: WindowEvent,
    ) {
        let waker = Arc::new(Waker).into();
        EVENT_LOOP.set(event_loop, || {
            WINDOW_EVENT.replace(Some(event));
            if let Poll::Ready(()) =
                self.fut.as_mut().poll(&mut Context::from_waker(&waker))
            {
                event_loop.exit();
            }
        });
    }
}

fn main() -> anyhow::Result<()> {
    use winit::event_loop::EventLoop;
    let fut = pin!(async {
        if let Err(err) = main_loop().await {
            eprintln!("{err:?}");
        }
    });
    let event_loop = EventLoop::new()?;
    event_loop.run_app(&mut App { fut })?;
    Ok(())
}
