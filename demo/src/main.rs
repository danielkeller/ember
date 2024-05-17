// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::collections::HashMap;
use std::io::Write;
use std::mem::size_of;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Context;
use inline_spirv::include_spirv;
use maia::vk;
use ultraviolet::{Mat4, Vec3};

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

fn pick_physical_device<'i>(
    phys: &[vk::PhysicalDevice<'i>],
) -> vk::PhysicalDevice<'i> {
    let discr = vk::PhysicalDeviceType::DISCRETE_GPU;
    let int = vk::PhysicalDeviceType::INTEGRATED_GPU;
    *phys
        .iter()
        .find(|p| p.properties().device_type == discr)
        .or_else(|| phys.iter().find(|p| p.properties().device_type == int))
        .unwrap_or(&phys[0])
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
    device: &vk::Device, queue: &mut vk::Queue, cmd_pool: &mut vk::CommandPool,
    src: &[u8], dst: &vk::Buffer, dst_stage_mask: vk::PipelineStageFlags,
    dst_access_mask: vk::AccessFlags,
) -> anyhow::Result<()> {
    let staging_buffer = vk::BufferWithoutMemory::new(
        &device,
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
    let memory = vk::DeviceMemory::new(&device, mem_size, host_mem)?;
    let staging_buffer = vk::Buffer::new(staging_buffer, &memory, 0)?;
    let mut memory = memory.map(0, src.len())?;
    memory.write_at(0).write_all(src)?;

    let mut recording_session = cmd_pool.reset()?;
    let mut transfer = recording_session.begin();
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
    let mut transfer = transfer.end()?;
    let fence = vk::Fence::new(device)?;
    queue.scope(|s| {
        s.submit(&mut [vk::Submit::Command(&mut transfer)]);
        s.submit(&mut [vk::Submit::Command(&mut transfer)]);
    });
    Ok(())
}

fn upload_image(
    device: &Arc<vk::Device>, queue: &mut vk::Queue, image: &Arc<vk::Image>,
    cmd_pool: &mut vk::CommandPool,
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
    let memory = vk::DeviceMemory::new(&device, mem_size, host_mem)?;
    let staging_buffer = vk::Buffer::new(staging_buffer, &memory, 0)?;
    let mut memory = memory.map(0, mem_size as usize)?;
    let mut writer = memory.write_at(0);
    for src in image_data.chunks_exact(3) {
        writer.write_all(src)?;
        writer.write_all(&[255])?;
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
    let mut transfer = transfer.end()?;
    let fence = vk::Fence::new(device)?;
    let pending_fence = queue.submit_with_fence(
        &mut [vk::SubmitInfo1 {
            commands: &mut [&mut transfer],
            ..Default::default()
        }],
        fence,
    )?;
    pending_fence.wait()?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    find_right_directory().context("Changing dirs")?;
    use winit::event_loop::EventLoop;
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop)?;
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

    let surf = maia::window::create_surface(&inst, &window)?;

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
    let device = vk::Device::new(
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
    let mut queue = device.take_queues()[0][0];

    let mut acquire_sem = vk::Semaphore::new(&device)?;
    let mut fence = Some(vk::Fence::new(&device)?);

    let window_size = window.inner_size();
    let mut swapchain_size =
        vk::Extent2D { width: window_size.width, height: window_size.height };
    let mut swapchain = Some(vk::ext::SwapchainKHR::new(
        &device,
        vk::CreateSwapchainFrom::Surface(surf),
        vk::SwapchainCreateInfoKHR {
            min_image_count: 3,
            image_format: vk::Format::B8G8R8A8_SRGB,
            image_extent: swapchain_size,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT
                | vk::ImageUsageFlags::TRANSFER_DST,
            ..Default::default()
        },
    )?);

    let mut cmd_pool = vk::CommandPool::new(&device, queue_family)?;

    let vertex_size = std::mem::size_of_val(&VERTEX_DATA);
    let index_size = std::mem::size_of_val(&INDEX_DATA);

    let device_mem = memory_type(
        &device.physical_device(),
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
    let vertex_buffer = vertex_buffer.allocate_memory(device_mem)?;
    upload_data(
        &device,
        &mut queue,
        &mut cmd_pool,
        bytemuck::bytes_of(&VERTEX_DATA),
        &vertex_buffer,
        vk::PipelineStageFlags::VERTEX_INPUT,
        vk::AccessFlags::VERTEX_ATTRIBUTE_READ,
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
    let index_buffer = index_buffer.allocate_memory(device_mem)?;
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
    let uniform_memory = vk::DeviceMemory::new(
        &device,
        uniform_buffer.memory_requirements().size,
        memory_type(
            &device.physical_device(),
            vk::MemoryPropertyFlags::DEVICE_LOCAL
                | vk::MemoryPropertyFlags::HOST_VISIBLE
                | vk::MemoryPropertyFlags::HOST_COHERENT,
        ),
    )?;
    let uniform_buffer = vk::Buffer::new(uniform_buffer, &uniform_memory, 0)?;
    let mut mapped = uniform_memory.map(0, size_of::<MVP>())?;

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
    let image = image.allocate_memory(device_mem)?;
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

    // TODO: rust-analyzer doesn't like this now and shaderc is annoying, maybe switching to WGSL and naga is nicer?
    let vertex_shader = vk::ShaderModule::new(
        &device,
        include_spirv!("shaders/triangle.vert", vert),
    )?;
    let fragment_shader = vk::ShaderModule::new(
        &device,
        include_spirv!("shaders/triangle.frag", frag),
    )?;

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
                immutable_samplers: vec![vk::Sampler::new(
                    &device,
                    &Default::default(),
                )?],
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
    let desc_set = Arc::new(desc_set);

    let pipeline_layout = vk::PipelineLayout::new(
        &device,
        Default::default(),
        vec![descriptor_set_layout.clone()],
        vec![],
    )?;

    let pipeline =
        vk::Pipeline::new_graphics(&vk::GraphicsPipelineCreateInfo {
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
            layout: &pipeline_layout,
            render_pass: &render_pass,
            subpass: 0,
            cache: None,
        })?;

    let mut framebuffers = HashMap::new();

    let begin = Instant::now();

    let mut redraw = move |draw_size: vk::Extent2D| -> anyhow::Result<()> {
        if draw_size != swapchain_size {
            swapchain_size = draw_size;
            framebuffers.clear();
            swapchain = Some(vk::ext::SwapchainKHR::new(
                &device,
                vk::CreateSwapchainFrom::OldSwapchain(
                    swapchain.take().unwrap(),
                ),
                vk::SwapchainCreateInfoKHR {
                    min_image_count: 3,
                    image_format: vk::Format::B8G8R8A8_SRGB,
                    image_extent: swapchain_size,
                    image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT
                        | vk::ImageUsageFlags::TRANSFER_DST,
                    ..Default::default()
                },
            )?);
        }

        let (img, _subopt) = swapchain
            .as_mut()
            .unwrap()
            .acquire_next_image(&mut acquire_sem, u64::MAX)?;

        if !framebuffers.contains_key(&img) {
            let img_view = vk::ImageView::new(
                &img,
                &vk::ImageViewCreateInfo {
                    format: vk::Format::B8G8R8A8_SRGB,
                    ..Default::default()
                },
            )?;
            let fb = vk::Framebuffer::new(
                &render_pass,
                Default::default(),
                vec![img_view],
                swapchain_size.into(),
            )?;
            let sem = vk::Semaphore::new(&device)?;
            framebuffers.insert(img.clone(), (fb, sem));
        }
        let (framebuffer, present_sem) = framebuffers.get_mut(&img).unwrap();

        let time = Instant::now().duration_since(begin);

        mapped.write_at(0).write_all(bytemuck::bytes_of(&MVP {
            model: Mat4::from_rotation_y(time.as_secs_f32() * 2.0),
            view: Mat4::look_at(
                Vec3::new(1., 1., 1.),
                Vec3::zero(),
                Vec3::new(0., 1., 0.),
            ),
            proj: ultraviolet::projection::perspective_infinite_z_vk(
                std::f32::consts::FRAC_PI_2,
                draw_size.width as f32 / draw_size.height as f32,
                0.1,
            ),
        }))?;

        let subpass = cmd_pool.allocate_secondary()?;
        let mut subpass = cmd_pool.begin_secondary(subpass, &render_pass, 0)?;
        subpass.set_viewport(&vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: draw_size.width as f32,
            height: draw_size.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        });
        subpass.bind_pipeline(&pipeline);
        subpass.bind_vertex_buffers(0, &[(&vertex_buffer, 0)])?;
        subpass.bind_index_buffer(&index_buffer, 0, vk::IndexType::UINT16)?;
        subpass.bind_descriptor_sets(
            vk::PipelineBindPoint::GRAPHICS,
            &pipeline_layout,
            0,
            &[&desc_set],
            &[],
        )?;
        subpass.draw_indexed(6, 1, 0, 0, 0)?;
        let mut subpass = subpass.end()?;

        let mut pass = cmd_pool.begin().begin_render_pass_secondary(
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
        pass.execute_commands(&mut [&mut subpass])?;
        let mut buf = pass.end()?.end()?;

        let pending_fence = queue.submit_with_fence(
            &mut [vk::SubmitInfo1 {
                wait: &mut [(
                    &mut acquire_sem,
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                )],
                commands: &mut [&mut buf],
                signal: &mut [present_sem],
            }],
            fence.take().unwrap(),
        )?;
        swapchain.as_mut().unwrap().present(&mut queue, &img, present_sem)?;
        fence = Some(pending_fence.wait()?);
        drop(buf);
        cmd_pool.reset(Default::default())?;
        Ok(())
    };

    event_loop.run(move |event, _, control_flow| {
        use winit::event::{Event, WindowEvent};
        use winit::event_loop::ControlFlow;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested, ..
            } => *control_flow = ControlFlow::Exit,
            Event::MainEventsCleared => {
                let window_size = window.inner_size();
                let window_size = vk::Extent2D {
                    width: window_size.width,
                    height: window_size.height,
                };
                if let Err(e) = redraw(window_size) {
                    println!("{:?}", e);
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => (),
        }
    })
}
