// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::device::Device;
use crate::enums::Bool;
use crate::enums::*;
use crate::ffi::*;
use crate::instance::Instance;
use crate::types::*;
use std::ffi::c_void;
use std::mem::transmute;
use std::mem::MaybeUninit;

#[link(name = "vulkan")]
extern "system" {
    fn vkGetInstanceProcAddr(
        instance: Option<Ref<VkInstance>>, name: Str<'_>,
    ) -> Option<NonNull<c_void>>;
}

pub unsafe fn vk_create_instance() -> unsafe extern "system" fn(
    &'_ InstanceCreateInfo<'_>,
    Option<&'_ AllocationCallbacks>,
    &mut Option<Handle<VkInstance>>,
) -> VkResult {
    transmute(load(None, "vkCreateInstance\0"))
}

pub unsafe fn vk_enumerate_instance_extension_properties(
) -> unsafe extern "system" fn(
    Option<Str<'_>>,
    &mut u32,
    Option<ArrayMut<MaybeUninit<ExtensionProperties>>>,
) -> VkResult {
    transmute(load(None, "vkEnumerateInstanceExtensionProperties\0"))
}

pub struct InstanceFn {
    pub destroy_instance: unsafe extern "system" fn(
        Mut<VkInstance>,
        Option<&'_ AllocationCallbacks>,
    ),
    pub enumerate_physical_devices: unsafe extern "system" fn(
        Ref<VkInstance>,
        &mut u32,
        Option<ArrayMut<MaybeUninit<Ref<VkPhysicalDevice>>>>,
    ) -> VkResult,
    pub get_physical_device_properties: unsafe extern "system" fn(
        Ref<VkPhysicalDevice>,
        &mut MaybeUninit<PhysicalDeviceProperties>,
    ),
    pub get_physical_device_queue_family_properties:
        unsafe extern "system" fn(
            Ref<VkPhysicalDevice>,
            &mut u32,
            Option<ArrayMut<MaybeUninit<QueueFamilyProperties>>>,
        ),
    pub get_physical_device_memory_properties: unsafe extern "system" fn(
        Ref<VkPhysicalDevice>,
        &mut PhysicalDeviceMemoryProperties,
    ),
    pub enumerate_device_extension_properties:
        unsafe extern "system" fn(
            Ref<VkPhysicalDevice>,
            Option<Str<'_>>,
            &mut u32,
            Option<ArrayMut<MaybeUninit<ExtensionProperties>>>,
        ) -> VkResult,
    pub create_device: unsafe extern "system" fn(
        Ref<VkPhysicalDevice>,
        &'_ DeviceCreateInfo,
        Option<&'_ AllocationCallbacks>,
        &mut Option<Handle<VkDevice>>,
    ) -> VkResult,
    pub get_device_proc_addr:
        unsafe extern "system" fn(
            Ref<VkDevice>,
            name: Str<'_>,
        ) -> Option<NonNull<c_void>>,
}

impl InstanceFn {
    pub fn new(inst: Ref<VkInstance>) -> Self {
        let inst = Some(inst);
        unsafe {
            Self {
                destroy_instance: transmute(load(inst, "vkDestroyInstance\0")),
                create_device: transmute(load(inst, "vkCreateDevice\0")),
                get_physical_device_properties: transmute(load(
                    inst,
                    "vkGetPhysicalDeviceProperties\0",
                )),
                get_physical_device_queue_family_properties: transmute(load(
                    inst,
                    "vkGetPhysicalDeviceQueueFamilyProperties\0",
                )),
                get_physical_device_memory_properties: transmute(load(
                    inst,
                    "vkGetPhysicalDeviceMemoryProperties\0",
                )),
                enumerate_device_extension_properties: transmute(load(
                    inst,
                    "vkEnumerateDeviceExtensionProperties\0",
                )),
                get_device_proc_addr: transmute(load(
                    inst,
                    "vkGetDeviceProcAddr\0",
                )),
                enumerate_physical_devices: transmute(load(
                    inst,
                    "vkEnumeratePhysicalDevices\0",
                )),
            }
        }
    }
}

/// Load instance function. Panics if the string is not null-terminated or the
/// function was not found.
fn load(instance: Option<Ref<VkInstance>>, name: &str) -> NonNull<c_void> {
    let ptr =
        unsafe { vkGetInstanceProcAddr(instance, name.try_into().unwrap()) };
    ptr.unwrap_or_else(|| {
        panic!("Could not load {:?}", &name[0..name.len() - 1])
    })
}

#[allow(clippy::type_complexity)]
pub struct DeviceFn {
    pub destroy_device: unsafe extern "system" fn(
        Mut<VkDevice>,
        Option<&'_ AllocationCallbacks>,
    ),
    pub device_wait_idle: unsafe extern "system" fn(
        // Technically not ext. sync. on the device, but on the queues. But
        // this is safer because the queues borrow the device.
        Mut<VkDevice>,
    ) -> VkResult,
    pub get_device_queue: unsafe extern "system" fn(
        Ref<VkDevice>,
        u32,
        u32,
        &mut Option<Handle<VkQueue>>,
    ),
    pub queue_submit: unsafe extern "system" fn(
        Mut<VkQueue>,
        u32,
        Option<Array<VkSubmitInfo<Null>>>,
        Option<Mut<VkFence>>,
    ) -> VkResult,
    pub queue_wait_idle: unsafe extern "system" fn(Mut<VkQueue>) -> VkResult,
    pub allocate_memory: unsafe extern "system" fn(
        Ref<VkDevice>,
        &MemoryAllocateInfo,
        Option<&'_ AllocationCallbacks>,
        &mut Option<Handle<VkDeviceMemory>>,
    ) -> VkResult,
    pub map_memory: unsafe extern "system" fn(
        Ref<VkDevice>,
        Mut<VkDeviceMemory>,
        u64,
        u64,
        MemoryMapFlags,
        &mut *mut u8,
    ) -> VkResult,
    pub unmap_memory:
        unsafe extern "system" fn(Ref<VkDevice>, Mut<VkDeviceMemory>),
    pub free_memory: unsafe extern "system" fn(
        Ref<VkDevice>,
        Mut<VkDeviceMemory>,
        Option<&'_ AllocationCallbacks>,
    ),
    pub create_fence: unsafe extern "system" fn(
        Ref<VkDevice>,
        &FenceCreateInfo,
        Option<&'_ AllocationCallbacks>,
        &mut Option<Handle<VkFence>>,
    ) -> VkResult,
    pub destroy_fence: unsafe extern "system" fn(
        Ref<VkDevice>,
        Mut<VkFence>,
        Option<&'_ AllocationCallbacks>,
    ),
    pub wait_for_fences: unsafe extern "system" fn(
        Ref<VkDevice>,
        u32,
        Array<Ref<VkFence>>,
        Bool,
        u64,
    ) -> VkResult,
    pub reset_fences: unsafe extern "system" fn(
        Ref<VkDevice>,
        u32,
        Array<Mut<VkFence>>,
    ) -> VkResult,
    pub create_semaphore: unsafe extern "system" fn(
        Ref<VkDevice>,
        &SemaphoreCreateInfo,
        Option<&'_ AllocationCallbacks>,
        &mut Option<Handle<VkSemaphore>>,
    ) -> VkResult,
    pub destroy_semaphore: unsafe extern "system" fn(
        Ref<VkDevice>,
        Mut<VkSemaphore>,
        Option<&'_ AllocationCallbacks>,
    ),
    pub create_buffer: unsafe extern "system" fn(
        Ref<VkDevice>,
        &BufferCreateInfo,
        Option<&'_ AllocationCallbacks>,
        &mut Option<Handle<VkBuffer>>,
    ) -> VkResult,
    pub destroy_buffer: unsafe extern "system" fn(
        Ref<VkDevice>,
        Mut<VkBuffer>,
        Option<&'_ AllocationCallbacks>,
    ),
    pub create_image: unsafe extern "system" fn(
        Ref<VkDevice>,
        &ImageCreateInfo,
        Option<&'_ AllocationCallbacks>,
        &mut Option<Handle<VkImage>>,
    ) -> VkResult,
    pub destroy_image: unsafe extern "system" fn(
        Ref<VkDevice>,
        Mut<VkImage>,
        Option<&'_ AllocationCallbacks>,
    ),
    pub get_buffer_memory_requirements: unsafe extern "system" fn(
        Ref<VkDevice>,
        Ref<VkBuffer>,
        &mut MemoryRequirements,
    ),
    pub get_image_memory_requirements: unsafe extern "system" fn(
        Ref<VkDevice>,
        Ref<VkImage>,
        &mut MemoryRequirements,
    ),
    pub bind_buffer_memory: unsafe extern "system" fn(
        Ref<VkDevice>,
        Mut<VkBuffer>,
        Ref<VkDeviceMemory>,
        u64,
    ) -> VkResult,
    pub bind_image_memory: unsafe extern "system" fn(
        Ref<VkDevice>,
        Mut<VkImage>,
        Ref<VkDeviceMemory>,
        u64,
    ) -> VkResult,
    pub create_image_view: unsafe extern "system" fn(
        Ref<VkDevice>,
        &VkImageViewCreateInfo,
        Option<&'_ AllocationCallbacks>,
        &mut Option<Handle<VkImageView>>,
    ) -> VkResult,
    pub destroy_image_view: unsafe extern "system" fn(
        Ref<VkDevice>,
        Mut<VkImageView>,
        Option<&'_ AllocationCallbacks>,
    ),
    pub create_shader_module: unsafe extern "system" fn(
        Ref<VkDevice>,
        &VkShaderModuleCreateInfo,
        Option<&'_ AllocationCallbacks>,
        &mut Option<Handle<VkShaderModule>>,
    ) -> VkResult,
    pub destroy_shader_module: unsafe extern "system" fn(
        Ref<VkDevice>,
        Mut<VkShaderModule>,
        Option<&'_ AllocationCallbacks>,
    ),
    pub create_pipeline_cache: unsafe extern "system" fn(
        Ref<VkDevice>,
        &PipelineCacheCreateInfo,
        Option<&'_ AllocationCallbacks>,
        &mut Option<Handle<VkPipelineCache>>,
    ) -> VkResult,
    pub destroy_pipeline_cache: unsafe extern "system" fn(
        Ref<VkDevice>,
        Mut<VkPipelineCache>,
        Option<&'_ AllocationCallbacks>,
    ),
    pub get_pipeline_cache_data: unsafe extern "system" fn(
        Ref<VkDevice>,
        Ref<VkPipelineCache>,
        &mut usize,
        Option<ArrayMut<MaybeUninit<u8>>>,
    ) -> VkResult,
    pub create_framebuffer: unsafe extern "system" fn(
        Ref<VkDevice>,
        &VkFramebufferCreateInfo,
        Option<&'_ AllocationCallbacks>,
        &mut Option<Handle<VkFramebuffer>>,
    ) -> VkResult,
    pub destroy_framebuffer: unsafe extern "system" fn(
        Ref<VkDevice>,
        Mut<VkFramebuffer>,
        Option<&'_ AllocationCallbacks>,
    ),
    pub create_render_pass: unsafe extern "system" fn(
        Ref<VkDevice>,
        &RenderPassCreateInfo,
        Option<&'_ AllocationCallbacks>,
        &mut Option<Handle<VkRenderPass>>,
    ) -> VkResult,
    pub destroy_render_pass: unsafe extern "system" fn(
        Ref<VkDevice>,
        Mut<VkRenderPass>,
        Option<&'_ AllocationCallbacks>,
    ),
    pub create_descriptor_set_layout: unsafe extern "system" fn(
        Ref<VkDevice>,
        &VkDescriptorSetLayoutCreateInfo,
        Option<&'_ AllocationCallbacks>,
        &mut Option<Handle<VkDescriptorSetLayout>>,
    )
        -> VkResult,
    pub destroy_descriptor_set_layout: unsafe extern "system" fn(
        Ref<VkDevice>,
        Mut<VkDescriptorSetLayout>,
        Option<&'_ AllocationCallbacks>,
    ),
    pub create_descriptor_pool: unsafe extern "system" fn(
        Ref<VkDevice>,
        &DescriptorPoolCreateInfo,
        Option<&'_ AllocationCallbacks>,
        &mut Option<Handle<VkDescriptorPool>>,
    ) -> VkResult,
    pub destroy_descriptor_pool: unsafe extern "system" fn(
        Ref<VkDevice>,
        Mut<VkDescriptorPool>,
        Option<&'_ AllocationCallbacks>,
    ),
    pub reset_descriptor_pool: unsafe extern "system" fn(
        Ref<VkDevice>,
        Mut<VkDescriptorPool>,
        DescriptorPoolResetFlags,
    ) -> VkResult,
    pub allocate_descriptor_sets: unsafe extern "system" fn(
        Ref<VkDevice>,
        &DescriptorSetAllocateInfo,
        ArrayMut<MaybeUninit<Handle<VkDescriptorSet>>>,
    ) -> VkResult,
    pub update_descriptor_sets: unsafe extern "system" fn(
        Ref<VkDevice>,
        u32,
        Option<Array<VkWriteDescriptorSet>>,
        u32,
        Option<Array<VkCopyDescriptorSet>>,
    ),
    pub create_pipeline_layout: unsafe extern "system" fn(
        Ref<VkDevice>,
        &PipelineLayoutCreateInfo,
        Option<&'_ AllocationCallbacks>,
        &mut Option<Handle<VkPipelineLayout>>,
    ) -> VkResult,
    pub destroy_pipeline_layout: unsafe extern "system" fn(
        Ref<VkDevice>,
        Mut<VkPipelineLayout>,
        Option<&'_ AllocationCallbacks>,
    ),
    pub create_sampler: unsafe extern "system" fn(
        Ref<VkDevice>,
        &SamplerCreateInfo,
        Option<&'_ AllocationCallbacks>,
        &mut Option<Handle<VkSampler>>,
    ) -> VkResult,
    pub destroy_sampler: unsafe extern "system" fn(
        Ref<VkDevice>,
        Mut<VkSampler>,
        Option<&'_ AllocationCallbacks>,
    ),
    pub create_graphics_pipelines: unsafe extern "system" fn(
        Ref<VkDevice>,
        Option<Ref<VkPipelineCache>>,
        u32,
        Array<VkGraphicsPipelineCreateInfo>,
        Option<&'_ AllocationCallbacks>,
        ArrayMut<MaybeUninit<Handle<VkPipeline>>>,
    ) -> VkResult,
    pub create_compute_pipelines: unsafe extern "system" fn(
        Ref<VkDevice>,
        Option<Ref<VkPipelineCache>>,
        u32,
        Array<ComputePipelineCreateInfo>,
        Option<&'_ AllocationCallbacks>,
        ArrayMut<MaybeUninit<Handle<VkPipeline>>>,
    ) -> VkResult,
    pub destroy_pipeline: unsafe extern "system" fn(
        Ref<VkDevice>,
        Mut<VkPipeline>,
        Option<&'_ AllocationCallbacks>,
    ),
    pub create_command_pool: unsafe extern "system" fn(
        Ref<VkDevice>,
        &CommandPoolCreateInfo,
        Option<&'_ AllocationCallbacks>,
        &mut Option<Handle<VkCommandPool>>,
    ) -> VkResult,
    pub destroy_command_pool: unsafe extern "system" fn(
        Ref<VkDevice>,
        Mut<VkCommandPool>,
        Option<&'_ AllocationCallbacks>,
    ),
    pub reset_command_pool: unsafe extern "system" fn(
        Ref<VkDevice>,
        Mut<VkCommandPool>,
        CommandPoolResetFlags,
    ) -> VkResult,
    pub allocate_command_buffers: unsafe extern "system" fn(
        Ref<VkDevice>,
        &CommandBufferAllocateInfo<'_>,
        ArrayMut<MaybeUninit<Handle<VkCommandBuffer>>>,
    ) -> VkResult,
    pub free_command_buffers: unsafe extern "system" fn(
        Ref<VkDevice>,
        Mut<VkCommandPool>,
        u32,
        &Mut<VkCommandBuffer>,
    ),
    pub begin_command_buffer: unsafe extern "system" fn(
        Mut<VkCommandBuffer>,
        &CommandBufferBeginInfo,
    ) -> VkResult,
    pub end_command_buffer:
        unsafe extern "system" fn(Mut<VkCommandBuffer>) -> VkResult,
    pub cmd_fill_buffer: unsafe extern "system" fn(
        Mut<VkCommandBuffer>,
        Ref<VkBuffer>,
        u64,
        u64,
        u32,
    ),
    pub cmd_copy_buffer: unsafe extern "system" fn(
        Mut<VkCommandBuffer>,
        Ref<VkBuffer>,
        Ref<VkBuffer>,
        u32,
        Array<BufferCopy>,
    ),
    pub cmd_copy_buffer_to_image: unsafe extern "system" fn(
        Mut<VkCommandBuffer>,
        Ref<VkBuffer>,
        Ref<VkImage>,
        ImageLayout,
        u32,
        Array<BufferImageCopy>,
    ),
    pub cmd_blit_image: unsafe extern "system" fn(
        Mut<VkCommandBuffer>,
        Ref<VkImage>,
        ImageLayout,
        Ref<VkImage>,
        ImageLayout,
        u32,
        Array<ImageBlit>,
        Filter,
    ),
    pub cmd_clear_color_image: unsafe extern "system" fn(
        Mut<VkCommandBuffer>,
        Ref<VkImage>,
        ImageLayout,
        &ClearColorValue,
        u32,
        Array<ImageSubresourceRange>,
    ),
    pub cmd_pipeline_barrier: unsafe extern "system" fn(
        Mut<VkCommandBuffer>,
        PipelineStageFlags,
        PipelineStageFlags,
        DependencyFlags,
        u32,
        Option<Array<MemoryBarrier>>,
        u32,
        Option<Array<BufferMemoryBarrier>>,
        u32,
        Option<Array<ImageMemoryBarrier>>,
    ),
    pub cmd_begin_render_pass: unsafe extern "system" fn(
        Mut<VkCommandBuffer>,
        &RenderPassBeginInfo,
        SubpassContents,
    ),
    pub cmd_next_subpass:
        unsafe extern "system" fn(Mut<VkCommandBuffer>, SubpassContents),
    pub cmd_end_render_pass: unsafe extern "system" fn(Mut<VkCommandBuffer>),
    pub cmd_execute_commands: unsafe extern "system" fn(
        Mut<VkCommandBuffer>,
        u32,
        Array<Mut<VkCommandBuffer>>,
    ),
    pub cmd_bind_pipeline: unsafe extern "system" fn(
        Mut<VkCommandBuffer>,
        PipelineBindPoint,
        Ref<VkPipeline>,
    ),
    pub cmd_bind_vertex_buffers: unsafe extern "system" fn(
        Mut<VkCommandBuffer>,
        u32,
        u32,
        Array<Ref<VkBuffer>>,
        Array<u64>,
    ),
    pub cmd_bind_index_buffer: unsafe extern "system" fn(
        Mut<VkCommandBuffer>,
        Ref<VkBuffer>,
        u64,
        IndexType,
    ),
    pub cmd_bind_descriptor_sets: unsafe extern "system" fn(
        Mut<VkCommandBuffer>,
        PipelineBindPoint,
        Ref<VkPipelineLayout>,
        u32,
        u32,
        Option<Array<Ref<VkDescriptorSet>>>,
        u32,
        Option<Array<u32>>,
    ),
    pub cmd_push_constants: unsafe extern "system" fn(
        Mut<VkCommandBuffer>,
        Ref<VkPipelineLayout>,
        ShaderStageFlags,
        u32,
        u32,
        Array<u8>,
    ),
    pub cmd_set_viewport: unsafe extern "system" fn(
        Mut<VkCommandBuffer>,
        u32,
        u32,
        Array<Viewport>,
    ),
    pub cmd_set_scissor: unsafe extern "system" fn(
        Mut<VkCommandBuffer>,
        u32,
        u32,
        Array<Rect2D>,
    ),
    pub cmd_draw:
        unsafe extern "system" fn(Mut<VkCommandBuffer>, u32, u32, u32, u32),
    pub cmd_draw_indirect: unsafe extern "system" fn(
        Mut<VkCommandBuffer>,
        Ref<VkBuffer>,
        u64,
        u32,
        u32,
    ),
    pub cmd_draw_indexed: unsafe extern "system" fn(
        Mut<VkCommandBuffer>,
        u32,
        u32,
        u32,
        i32,
        u32,
    ),
    pub cmd_draw_indexed_indirect: unsafe extern "system" fn(
        Mut<VkCommandBuffer>,
        Ref<VkBuffer>,
        u64,
        u32,
        u32,
    ),
    pub cmd_dispatch:
        unsafe extern "system" fn(Mut<VkCommandBuffer>, u32, u32, u32),
    pub cmd_dispatch_indirect:
        unsafe extern "system" fn(Mut<VkCommandBuffer>, Ref<VkBuffer>, u64),
}

// Reduce indent
unsafe fn new_device_fn(inst: &Instance, device: Ref<VkDevice>) -> DeviceFn {
    let load = |name| inst.load(device, name);
    DeviceFn {
        destroy_device: transmute(load("vkDestroyDevice\0")),
        device_wait_idle: transmute(load("vkDeviceWaitIdle\0")),
        get_device_queue: transmute(load("vkGetDeviceQueue\0")),
        queue_submit: transmute(load("vkQueueSubmit\0")),
        queue_wait_idle: transmute(load("vkQueueWaitIdle\0")),
        allocate_memory: transmute(load("vkAllocateMemory\0")),
        map_memory: transmute(load("vkMapMemory\0")),
        unmap_memory: transmute(load("vkUnmapMemory\0")),
        free_memory: transmute(load("vkFreeMemory\0")),
        create_fence: transmute(load("vkCreateFence\0")),
        destroy_fence: transmute(load("vkDestroyFence\0")),
        wait_for_fences: transmute(load("vkWaitForFences\0")),
        reset_fences: transmute(load("vkResetFences\0")),
        create_semaphore: transmute(load("vkCreateSemaphore\0")),
        destroy_semaphore: transmute(load("vkDestroySemaphore\0")),
        create_buffer: transmute(load("vkCreateBuffer\0")),
        destroy_buffer: transmute(load("vkDestroyBuffer\0")),
        create_image: transmute(load("vkCreateImage\0")),
        destroy_image: transmute(load("vkDestroyImage\0")),
        get_buffer_memory_requirements: transmute(load(
            "vkGetBufferMemoryRequirements\0",
        )),
        get_image_memory_requirements: transmute(load(
            "vkGetImageMemoryRequirements\0",
        )),
        bind_buffer_memory: transmute(load("vkBindBufferMemory\0")),
        bind_image_memory: transmute(load("vkBindImageMemory\0")),
        create_image_view: transmute(load("vkCreateImageView\0")),
        destroy_image_view: transmute(load("vkDestroyImageView\0")),
        create_shader_module: transmute(load("vkCreateShaderModule\0")),
        destroy_shader_module: transmute(load("vkDestroyShaderModule\0")),
        create_pipeline_cache: transmute(load("vkCreatePipelineCache\0")),
        destroy_pipeline_cache: transmute(load("vkDestroyPipelineCache\0")),
        get_pipeline_cache_data: transmute(load("vkGetPipelineCacheData\0")),
        create_framebuffer: transmute(load("vkCreateFramebuffer\0")),
        destroy_framebuffer: transmute(load("vkDestroyFramebuffer\0")),
        create_render_pass: transmute(load("vkCreateRenderPass\0")),
        destroy_render_pass: transmute(load("vkDestroyRenderPass\0")),
        create_descriptor_set_layout: transmute(load(
            "vkCreateDescriptorSetLayout\0",
        )),
        destroy_descriptor_set_layout: transmute(load(
            "vkDestroyDescriptorSetLayout\0",
        )),
        create_descriptor_pool: transmute(load("vkCreateDescriptorPool\0")),
        destroy_descriptor_pool: transmute(load("vkDestroyDescriptorPool\0")),
        reset_descriptor_pool: transmute(load("vkResetDescriptorPool\0")),
        allocate_descriptor_sets: transmute(load("vkAllocateDescriptorSets\0")),
        update_descriptor_sets: transmute(load("vkUpdateDescriptorSets\0")),
        create_pipeline_layout: transmute(load("vkCreatePipelineLayout\0")),
        destroy_pipeline_layout: transmute(load("vkDestroyPipelineLayout\0")),
        create_sampler: transmute(load("vkCreateSampler\0")),
        destroy_sampler: transmute(load("vkDestroySampler\0")),
        create_graphics_pipelines: transmute(load(
            "vkCreateGraphicsPipelines\0",
        )),
        create_compute_pipelines: transmute(load("vkCreateComputePipelines\0")),
        destroy_pipeline: transmute(load("vkDestroyPipeline\0")),
        create_command_pool: transmute(load("vkCreateCommandPool\0")),
        destroy_command_pool: transmute(load("vkDestroyCommandPool\0")),
        reset_command_pool: transmute(load("vkResetCommandPool\0")),
        allocate_command_buffers: transmute(load("vkAllocateCommandBuffers\0")),
        free_command_buffers: transmute(load("vkFreeCommandBuffers\0")),
        begin_command_buffer: transmute(load("vkBeginCommandBuffer\0")),
        end_command_buffer: transmute(load("vkEndCommandBuffer\0")),
        cmd_fill_buffer: transmute(load("vkCmdFillBuffer\0")),
        cmd_copy_buffer: transmute(load("vkCmdCopyBuffer\0")),
        cmd_copy_buffer_to_image: transmute(load("vkCmdCopyBufferToImage\0")),
        cmd_blit_image: transmute(load("vkCmdBlitImage\0")),
        cmd_clear_color_image: transmute(load("vkCmdClearColorImage\0")),
        cmd_pipeline_barrier: transmute(load("vkCmdPipelineBarrier\0")),
        cmd_begin_render_pass: transmute(load("vkCmdBeginRenderPass\0")),
        cmd_next_subpass: transmute(load("vkCmdNextSubpass\0")),
        cmd_end_render_pass: transmute(load("vkCmdEndRenderPass\0")),
        cmd_execute_commands: transmute(load("vkCmdExecuteCommands\0")),
        cmd_bind_pipeline: transmute(load("vkCmdBindPipeline\0")),
        cmd_bind_vertex_buffers: transmute(load("vkCmdBindVertexBuffers\0")),
        cmd_bind_index_buffer: transmute(load("vkCmdBindIndexBuffer\0")),
        cmd_bind_descriptor_sets: transmute(load("vkCmdBindDescriptorSets\0")),
        cmd_push_constants: transmute(load("vkCmdPushConstants\0")),
        cmd_set_viewport: transmute(load("vkCmdSetViewport\0")),
        cmd_set_scissor: transmute(load("vkCmdSetScissor\0")),
        cmd_draw: transmute(load("vkCmdDraw\0")),
        cmd_draw_indirect: transmute(load("vkCmdDrawIndirect\0")),
        cmd_draw_indexed: transmute(load("vkCmdDrawIndexed\0")),
        cmd_draw_indexed_indirect: transmute(load(
            "vkCmdDrawIndexedIndirect\0",
        )),
        cmd_dispatch: transmute(load("vkCmdDispatch\0")),
        cmd_dispatch_indirect: transmute(load("vkCmdDispatchIndirect\0")),
    }
}

impl DeviceFn {
    pub fn new(inst: &Instance, device: Ref<VkDevice>) -> Self {
        unsafe { new_device_fn(inst, device) }
    }
}

impl Instance {
    /// Loads device function. Panics if the string is not null-terminated or the
    /// function was not found.
    fn load(&self, device: Ref<VkDevice>, name: &str) -> NonNull<c_void> {
        let ptr = unsafe {
            (self.fun.get_device_proc_addr)(device, name.try_into().unwrap())
        };
        ptr.unwrap_or_else(|| {
            panic!("Could not load {:?}", &name[0..name.len() - 1])
        })
    }

    /// Loads instance function. Panics if the string is not null-terminated or
    /// the function was not found.
    #[doc = crate::man_link!(vkGetInstanceProcAddr)]
    pub fn get_proc_addr(&self, name: &str) -> NonNull<c_void> {
        crate::load::load(Some(self.handle()), name)
    }
}

impl Device<'_> {
    /// Loads device function. Panics if the string is not null-terminated or the
    /// function was not found.
    #[doc = crate::man_link!(vkGetDeviceProcAddr)]
    pub fn get_proc_addr(&self, name: &str) -> NonNull<c_void> {
        self.instance().load(self.borrow(), name)
    }
}
