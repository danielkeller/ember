// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::enums::*;
use crate::error::VkResult;
use crate::ffi::*;
use std::fmt::Debug;
pub(crate) use std::sync::Arc;

#[doc = crate::man_link!(VK_DEFINE_HANDLE)]
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NonNullDispatchableHandle(NonNull<c_void>);
#[doc = crate::man_link!(VK_DEFINE_NON_DISPATCHABLE_HANDLE)]
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NonNullNonDispatchableHandle(std::num::NonZeroU64);

impl Debug for NonNullDispatchableHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.0))
    }
}

impl Debug for NonNullNonDispatchableHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:#x}", self.0))
    }
}

// This hides its pointer and is thus thread safe. Theoretically, if we actually
// used raw handle types anywhere they would be !Send + !Sync and the thread
// safety would be provided by Handle, Ref, and Mut. But this appears
// unneccesary.
unsafe impl Send for NonNullDispatchableHandle {}
unsafe impl Sync for NonNullDispatchableHandle {}
impl std::panic::UnwindSafe for NonNullDispatchableHandle {}
impl std::panic::RefUnwindSafe for NonNullDispatchableHandle {}

/// Owned Vulkan handle.
#[repr(transparent)]
#[derive(PartialEq, Eq, Hash)]
pub struct Handle<T> {
    _value: T,
}
impl<T: Debug> Debug for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self._value.fmt(f)
    }
}
impl<T: Copy> Handle<T> {
    /// Borrow the object
    pub fn borrow(&self) -> Ref<'_, T> {
        Ref { _value: self._value, _lt: PhantomData }
    }
    // TODO: I think it should be called get_mut?
    /// Mutably borrow the object
    pub fn borrow_mut(&mut self) -> Mut<'_, T> {
        Mut { _value: self._value, _lt: PhantomData }
    }
    /// Make a copy of the handle
    /// # Safety
    /// The caller must ensure that uses of the result are externally
    /// synchronized as defined by Vulkan.
    pub unsafe fn clone(&self) -> Self {
        Self { ..*self }
    }
    /// Mutably borrow the object from a shared reference
    /// # Safety
    /// The caller must ensure that uses of the result are externally
    /// synchronized as defined by Vulkan.
    pub unsafe fn borrow_mut_unchecked(&self) -> Mut<'_, T> {
        Mut { _value: self._value, _lt: PhantomData }
    }
}

/// Borrowed Vulkan handle. Has the same representation as a Vulkan handle but
/// carries a lifetime.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Ref<'a, T> {
    _value: T,
    _lt: PhantomData<&'a T>,
}
impl<T: Debug> Debug for Ref<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self._value.fmt(f)
    }
}

/// Mutably borrowed Vulkan handle. Has the same representation as a Vulkan handle but
/// carries a lifetime.
#[repr(transparent)]
pub struct Mut<'a, T> {
    _value: T,
    _lt: PhantomData<&'a T>,
}
impl<T: Debug> Debug for Mut<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self._value.fmt(f)
    }
}
impl<'a, T: Copy> Mut<'a, T> {
    /// Reborrow as a shared reference
    // Note that this cannot be used to extend 'a, since &'b Self<'a>
    // requires 'a: 'b
    pub fn reborrow(&self) -> Ref<'_, T> {
        Ref { _value: self._value, _lt: PhantomData }
    }
    /// Reborrow as a mutable reference
    // TODO: Is this the right name? See e.g. Pin.
    pub fn reborrow_mut(&mut self) -> Mut<'_, T> {
        Self { ..*self }
    }
    /// Reborrow as a mutable reference with a new lifetime.
    /// # Safety
    /// The caller must ensure that uses of the result are externally
    /// synchronized as defined by Vulkan. Note that this is more unsafe than
    /// the other unchecked borrows, since it allows the lifetime to be extended
    pub unsafe fn reborrow_mut_unchecked<'b>(&mut self) -> Mut<'b, T> {
        Mut { _value: self._value, _lt: PhantomData }
    }
}

macro_rules! raw_handle {
    ($name:ident($kind:ident)) => {
        #[repr(transparent)]
        #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        /// Raw Vulkan handle. Used either in owned form ([`Handle`]), borrowed
        /// form ([`Ref`]), or exclusive borrowed form ([`Mut`]).
        pub struct $name($kind);
    };
}

raw_handle!(VkInstance(NonNullDispatchableHandle));
raw_handle!(VkPhysicalDevice(NonNullDispatchableHandle));
raw_handle!(VkDevice(NonNullDispatchableHandle));
raw_handle!(VkQueue(NonNullDispatchableHandle));

raw_handle!(VkDeviceMemory(NonNullNonDispatchableHandle));
raw_handle!(VkSemaphore(NonNullNonDispatchableHandle));
raw_handle!(VkFence(NonNullNonDispatchableHandle));
raw_handle!(VkSampler(NonNullNonDispatchableHandle));
raw_handle!(VkDescriptorSetLayout(NonNullNonDispatchableHandle));
raw_handle!(VkDescriptorPool(NonNullNonDispatchableHandle));
raw_handle!(VkDescriptorSet(NonNullNonDispatchableHandle));
raw_handle!(VkPipelineLayout(NonNullNonDispatchableHandle));
raw_handle!(VkPipelineCache(NonNullNonDispatchableHandle));
raw_handle!(VkPipeline(NonNullNonDispatchableHandle));
raw_handle!(VkBuffer(NonNullNonDispatchableHandle));
raw_handle!(VkBufferView(NonNullNonDispatchableHandle));
raw_handle!(VkImage(NonNullNonDispatchableHandle));
raw_handle!(VkImageView(NonNullNonDispatchableHandle));
raw_handle!(VkFramebuffer(NonNullNonDispatchableHandle));
raw_handle!(VkRenderPass(NonNullNonDispatchableHandle));
raw_handle!(VkShaderModule(NonNullNonDispatchableHandle));
raw_handle!(VkCommandPool(NonNullNonDispatchableHandle));
raw_handle!(VkCommandBuffer(NonNullNonDispatchableHandle));
raw_handle!(VkSurfaceKHR(NonNullNonDispatchableHandle));
raw_handle!(VkSwapchainKHR(NonNullNonDispatchableHandle));

/// u32 with only one allowed value
macro_rules! structure_type {
    ($name: ident, $value: literal) => {
        #[repr(u32)]
        #[derive(Debug)]
        /// [Structure type](https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkStructureType.html) constant
        #[doc = concat!("(", stringify!($value), ")")]
        pub enum $name {
            #[doc = "Has the value "]
            #[doc = stringify!($value)]
            Value = $value,
        }
        impl Default for $name {
            fn default() -> Self {
                Self::Value
            }
        }
    };
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Extent2D {
    pub width: u32,
    pub height: u32,
}

impl Extent2D {
    /// Create a new `Extent2D` object
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Extent3D {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

impl Extent3D {
    /// Create a new `Extent3D` object
    pub fn new(width: u32, height: u32, depth: u32) -> Self {
        Self { width, height, depth }
    }
}

impl From<Extent2D> for Extent3D {
    fn from(e: Extent2D) -> Self {
        Self { width: e.width, height: e.height, depth: 1 }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Offset2D {
    pub x: i32,
    pub y: i32,
}

impl Offset2D {
    /// Create a new `Offeset2D` object
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Offset3D {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Offset3D {
    /// Create a new `Offeset3D` object
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Rect2D {
    pub offset: Offset2D,
    pub extent: Extent2D,
}

#[repr(C)]
#[derive(Clone, Copy)]
#[doc = crate::man_link!(VkClearColorValue)]
pub union ClearColorValue {
    pub f32: [f32; 4],
    pub i32: [i32; 4],
    pub u32: [u32; 4],
}
impl Default for ClearColorValue {
    /// Black for any format
    fn default() -> Self {
        Self { u32: [0, 0, 0, 0] }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
#[doc = crate::man_link!(VkClearDepthStencilValue)]
pub struct ClearDepthStencilValue {
    pub depth: f32,
    pub stencil: u32,
}

// Safety: If the user initializes the wrong member, leaving the struct partly
// uninitialized, the uninitialized value is never read by *rust*, only passed
// over the ffi.

#[repr(C)]
#[derive(Clone, Copy)]
#[doc = crate::man_link!(VkClearValue)]
pub union ClearValue {
    pub color: ClearColorValue,
    pub depth_stencil: ClearDepthStencilValue,
}

impl Default for ClearValue {
    /// Black for color, zero for depth/stencil
    fn default() -> Self {
        Self { color: Default::default() }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
#[doc = crate::man_link!(VkImageSubresourceRange)]
pub struct ImageSubresourceRange {
    pub aspect_mask: ImageAspectFlags,
    pub base_mip_level: u32,
    pub level_count: u32,
    pub base_array_layer: u32,
    pub layer_count: u32,
}

impl Default for ImageSubresourceRange {
    /// The entirety of a color image
    fn default() -> Self {
        Self {
            aspect_mask: ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: u32::MAX,
            base_array_layer: 0,
            layer_count: u32::MAX,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
#[doc = crate::man_link!(VkImageSubresourceLayers)]
pub struct ImageSubresourceLayers {
    pub aspect_mask: ImageAspectFlags,
    pub mip_level: u32,
    pub base_array_layer: u32,
    pub layer_count: u32,
}

impl Default for ImageSubresourceLayers {
    /// The first level and layer of a color image
    fn default() -> Self {
        Self {
            aspect_mask: ImageAspectFlags::COLOR,
            mip_level: 0,
            base_array_layer: 0,
            layer_count: 1,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
#[doc = crate::man_link!(VkImageBlit)]
pub struct ImageBlit {
    pub src_subresource: ImageSubresourceLayers,
    pub src_offsets: [Offset3D; 2],
    pub dst_subresource: ImageSubresourceLayers,
    pub dst_offsets: [Offset3D; 2],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
#[doc = crate::man_link!(VkComponentMapping)]
pub struct ComponentMapping {
    pub r: ComponentSwizzle,
    pub g: ComponentSwizzle,
    pub b: ComponentSwizzle,
    pub a: ComponentSwizzle,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
#[doc = crate::man_link!(VkViewport)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub min_depth: f32,
    pub max_depth: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
#[doc = crate::man_link!(VkMemoryType)]
pub struct MemoryType {
    pub property_flags: MemoryPropertyFlags,
    pub heap_index: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
#[doc = crate::man_link!(VkMemoryHeap)]
pub struct MemoryHeap {
    pub size: u64,
    pub flags: MemoryHeapFlags,
}

#[repr(C)]
#[derive(Debug, Default)]
#[doc = crate::man_link!(VkPhysicalDeviceMemoryProperties)]
pub struct PhysicalDeviceMemoryProperties {
    pub memory_types: InlineSlice<MemoryType, 32>,
    pub memory_heaps: InlineSlice<MemoryHeap, 16>,
}

#[repr(C)]
#[derive(Debug, Default)]
#[doc = crate::man_link!(VkMemoryRequirements)]
pub struct MemoryRequirements {
    pub size: u64,
    pub alignment: u64,
    pub memory_type_bits: u32,
}

impl MemoryRequirements {
    pub(crate) fn clear_host_visible_types(
        &mut self, props: &PhysicalDeviceMemoryProperties,
    ) {
        for (i, ty) in props.memory_types.iter().enumerate() {
            if ty.property_flags.contains(MemoryPropertyFlags::HOST_VISIBLE) {
                self.memory_type_bits &= !(1 << i);
            }
        }
    }
}

#[repr(C)]
#[derive(Debug, Default)]
#[doc = crate::man_link!(VkBufferCopy)]
pub struct BufferCopy {
    pub src_offset: u64,
    pub dst_offset: u64,
    pub size: u64,
}

#[repr(C)]
#[derive(Debug, Default)]
#[doc = crate::man_link!(VkBufferImageCopy)]
pub struct BufferImageCopy {
    pub buffer_offset: u64,
    pub buffer_row_length: u32,
    pub buffer_image_height: u32,
    pub image_subresource: ImageSubresourceLayers,
    pub image_offset: Offset3D,
    pub image_extent: Extent3D,
}

/// Not implemented
pub enum AllocationCallbacks {}

#[repr(C)]
#[derive(Debug, Default)]
#[doc = crate::man_link!(VkInstanceCreateInfo)]
pub struct InstanceCreateInfo<'a, Next = Null, AINext = Null> {
    pub stype: InstanceCreateInfoType,
    pub next: Next,
    pub flags: InstanceCreateFlags,
    pub application_info: Option<&'a ApplicationInfo<'a, AINext>>,
    pub enabled_layer_names: Slice<'a, Str<'a>>,
    pub enabled_extension_names: Slice<'a, Str<'a>>,
}
structure_type!(InstanceCreateInfoType, 1);

#[repr(C)]
#[derive(Debug, Default)]
#[doc = crate::man_link!(VkApplicationInfo)]
pub struct ApplicationInfo<'a, Next> {
    pub stype: ApplicationInfoType,
    pub next: Next,
    pub application_name: Str<'a>,
    pub application_version: u32,
    pub engine_name: Str<'a>,
    pub engine_version: u32,
    pub api_version: u32,
}
structure_type!(ApplicationInfoType, 0);

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkExtensionProperties)]
pub struct ExtensionProperties {
    pub extension_name: CharArray<MAX_EXTENSION_NAME_SIZE>,
    pub spec_version: u32,
}

pub const MAX_EXTENSION_NAME_SIZE: usize = 256;

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkPhysicalDeviceProperties)]
pub struct PhysicalDeviceProperties {
    pub api_version: u32,
    pub driver_version: u32,
    pub vendor_id: u32,
    pub device_id: u32,
    pub device_type: PhysicalDeviceType,
    pub device_name: CharArray<MAX_PHYSICAL_DEVICE_NAME_SIZE>,
    pub pipeline_cache_uuid: UUID,
    pub limits: PhysicalDeviceLimits,
    pub sparse_properties: PhysicalDeviceSparseProperties,
}

pub const MAX_PHYSICAL_DEVICE_NAME_SIZE: usize = 256;

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkPhysicalDeviceLimits)]
pub struct PhysicalDeviceLimits {
    pub max_image_dimension_1d: u32,
    pub max_image_dimension_2d: u32,
    pub max_image_dimension_3d: u32,
    pub max_image_dimension_cube: u32,
    pub max_image_array_layers: u32,
    pub max_texel_buffer_elements: u32,
    pub max_uniform_buffer_range: u32,
    pub max_storage_buffer_range: u32,
    pub max_push_constants_size: u32,
    pub max_memory_allocation_count: u32,
    pub max_sampler_allocation_count: u32,
    pub buffer_image_granularity: u64,
    pub sparse_address_space_size: u64,
    pub max_bound_descriptor_sets: u32,
    pub max_per_stage_descriptor_samplers: u32,
    pub max_per_stage_descriptor_uniform_buffers: u32,
    pub max_per_stage_descriptor_storage_buffers: u32,
    pub max_per_stage_descriptor_sampled_images: u32,
    pub max_per_stage_descriptor_storage_images: u32,
    pub max_per_stage_descriptor_input_attachments: u32,
    pub max_per_stage_resources: u32,
    pub max_descriptor_set_samplers: u32,
    pub max_descriptor_set_uniform_buffers: u32,
    pub max_descriptor_set_uniform_buffers_dynamic: u32,
    pub max_descriptor_set_storage_buffers: u32,
    pub max_descriptor_set_storage_buffers_dynamic: u32,
    pub max_descriptor_set_sampled_images: u32,
    pub max_descriptor_set_storage_images: u32,
    pub max_descriptor_set_input_attachments: u32,
    pub max_vertex_input_attributes: u32,
    pub max_vertex_input_bindings: u32,
    pub max_vertex_input_attribute_offset: u32,
    pub max_vertex_input_binding_stride: u32,
    pub max_vertex_output_components: u32,
    pub max_tessellation_generation_level: u32,
    pub max_tessellation_patch_size: u32,
    pub max_tessellation_control_per_vertex_input_components: u32,
    pub max_tessellation_control_per_vertex_output_components: u32,
    pub max_tessellation_control_per_patch_output_components: u32,
    pub max_tessellation_control_total_output_components: u32,
    pub max_tessellation_evaluation_input_components: u32,
    pub max_tessellation_evaluation_output_components: u32,
    pub max_geometry_shader_invocations: u32,
    pub max_geometry_input_components: u32,
    pub max_geometry_output_components: u32,
    pub max_geometry_output_vertices: u32,
    pub max_geometry_total_output_components: u32,
    pub max_fragment_input_components: u32,
    pub max_fragment_output_attachments: u32,
    pub max_fragment_dual_src_attachments: u32,
    pub max_fragment_combined_output_resources: u32,
    pub max_compute_shared_memory_size: u32,
    pub max_compute_work_group_count: [u32; 3],
    pub max_compute_work_group_invocations: u32,
    pub max_compute_work_group_size: [u32; 3],
    pub sub_pixel_precision_bits: u32,
    pub sub_texel_precision_bits: u32,
    pub mipmap_precision_bits: u32,
    pub max_draw_indexed_index_value: u32,
    pub max_draw_indirect_count: u32,
    pub max_sampler_lod_bias: f32,
    pub max_sampler_anisotropy: f32,
    pub max_viewports: u32,
    pub max_viewport_dimensions: [u32; 2],
    pub viewport_bounds_range: [f32; 2],
    pub viewport_sub_pixel_bits: u32,
    pub min_memory_map_alignment: usize,
    pub min_texel_buffer_offset_alignment: u64,
    pub min_uniform_buffer_offset_alignment: u64,
    pub min_storage_buffer_offset_alignment: u64,
    pub min_texel_offset: i32,
    pub max_texel_offset: u32,
    pub min_texel_gather_offset: i32,
    pub max_texel_gather_offset: u32,
    pub min_interpolation_offset: f32,
    pub max_interpolation_offset: f32,
    pub sub_pixel_interpolation_offset_bits: u32,
    pub max_framebuffer_width: u32,
    pub max_framebuffer_height: u32,
    pub max_framebuffer_layers: u32,
    pub framebuffer_color_sample_counts: SampleCountFlags,
    pub framebuffer_depth_sample_counts: SampleCountFlags,
    pub framebuffer_stencil_sample_counts: SampleCountFlags,
    pub framebuffer_no_attachments_sample_counts: SampleCountFlags,
    pub max_color_attachments: u32,
    pub sampled_image_color_sample_counts: SampleCountFlags,
    pub sampled_image_integer_sample_counts: SampleCountFlags,
    pub sampled_image_depth_sample_counts: SampleCountFlags,
    pub sampled_image_stencil_sample_counts: SampleCountFlags,
    pub storage_image_sample_counts: SampleCountFlags,
    pub max_sample_mask_words: u32,
    pub timestamp_compute_and_graphics: Bool,
    pub timestamp_period: f32,
    pub max_clip_distances: u32,
    pub max_cull_distances: u32,
    pub max_combined_clip_and_cull_distances: u32,
    pub discrete_queue_priorities: u32,
    pub point_size_range: [f32; 2],
    pub line_width_range: [f32; 2],
    pub point_size_granularity: f32,
    pub line_width_granularity: f32,
    pub strict_lines: Bool,
    pub standard_sample_locations: Bool,
    pub optimal_buffer_copy_offset_alignment: u64,
    pub optimal_buffer_copy_row_pitch_alignment: u64,
    pub non_coherent_atom_size: u64,
}

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkPhysicalDeviceSparseProperties)]
pub struct PhysicalDeviceSparseProperties {
    pub residency_standard_2d_block_shape: Bool,
    pub residency_standard_2d_multisample_block_shape: Bool,
    pub residency_standard_3d_block_shape: Bool,
    pub residency_aligned_mip_size: Bool,
    pub residency_non_resident_strict: Bool,
}

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkQueueFamilyProperties)]
pub struct QueueFamilyProperties {
    pub queue_flags: QueueFlags,
    pub queue_count: u32,
    pub timestamp_valid_bits: u32,
    pub min_image_transfer_granularity: Extent3D,
}

#[repr(C)]
#[derive(Debug, Default)]
#[doc = crate::man_link!(VkDeviceCreateInfo)]
pub struct DeviceCreateInfo<'a, Next = Null> {
    pub stype: DeviceCreateInfoType,
    pub next: Next,
    pub flags: DeviceCreateFlags,
    pub queue_create_infos: Slice_<'a, DeviceQueueCreateInfo<'a>>,
    pub enabled_layer_names: Slice<'a, Str<'a>>,
    pub enabled_extension_names: Slice<'a, Str<'a>>,
    pub enabled_features: Option<&'a PhysicalDeviceFeatures>,
}
structure_type!(DeviceCreateInfoType, 3);

#[repr(C)]
#[derive(Debug, Default)]
#[doc = crate::man_link!(VkDeviceQueueCreateInfo)]
pub struct DeviceQueueCreateInfo<'a, Next = Null> {
    pub stype: DeviceQueueCreateInfoType,
    pub next: Next,
    pub flags: DeviceQueueCreateFlags,
    pub queue_family_index: u32,
    pub queue_priorities: Slice<'a, f32>,
}
structure_type!(DeviceQueueCreateInfoType, 2);

#[repr(C)]
#[derive(Clone, Default, Debug)]
#[doc = crate::man_link!(VkPhysicalDeviceFeatures)]
pub struct PhysicalDeviceFeatures {
    pub robust_buffer_access: Bool,
    pub full_draw_index_uint32: Bool,
    pub image_cube_array: Bool,
    pub independent_blend: Bool,
    pub geometry_shader: Bool,
    pub tessellation_shader: Bool,
    pub sample_rate_shading: Bool,
    pub dual_src_blend: Bool,
    pub logic_op: Bool,
    pub multi_draw_indirect: Bool,
    pub draw_indirect_first_instance: Bool,
    pub depth_clamp: Bool,
    pub depth_bias_clamp: Bool,
    pub fill_mode_non_solid: Bool,
    pub depth_bounds: Bool,
    pub wide_lines: Bool,
    pub large_points: Bool,
    pub alpha_to_one: Bool,
    pub multi_viewport: Bool,
    pub sampler_anisotropy: Bool,
    pub texture_compression_etc2: Bool,
    pub texture_compression_astc_ldr: Bool,
    pub texture_compression_bc: Bool,
    pub occlusion_query_precise: Bool,
    pub pipeline_statistics_query: Bool,
    pub vertex_pipeline_stores_and_atomics: Bool,
    pub fragment_stores_and_atomics: Bool,
    pub shader_tessellation_and_geometry_point_size: Bool,
    pub shader_image_gather_extended: Bool,
    pub shader_storage_image_extended_formats: Bool,
    pub shader_storage_image_multisample: Bool,
    pub shader_storage_image_read_without_format: Bool,
    pub shader_storage_image_write_without_format: Bool,
    pub shader_uniform_buffer_array_dynamic_indexing: Bool,
    pub shader_sampled_image_array_dynamic_indexing: Bool,
    pub shader_storage_buffer_array_dynamic_indexing: Bool,
    pub shader_storage_image_array_dynamic_indexing: Bool,
    pub shader_clip_distance: Bool,
    pub shader_cull_distance: Bool,
    pub shader_float64: Bool,
    pub shader_int64: Bool,
    pub shader_int16: Bool,
    pub shader_resource_residency: Bool,
    pub shader_resource_min_lod: Bool,
    pub sparse_binding: Bool,
    pub sparse_residency_buffer: Bool,
    pub sparse_residency_image_2d: Bool,
    pub sparse_residency_image_3d: Bool,
    pub sparse_residency2_samples: Bool,
    pub sparse_residency4_samples: Bool,
    pub sparse_residency8_samples: Bool,
    pub sparse_residency16_samples: Bool,
    pub sparse_residency_aliased: Bool,
    pub variable_multisample_rate: Bool,
    pub inherited_queries: Bool,
}

#[repr(C)]
#[derive(Debug, Default)]
#[doc = crate::man_link!(VkSubmitInfo)]
pub struct VkSubmitInfo<'a, Next = Null> {
    pub stype: SubmitInfoType,
    pub next: Next,
    pub wait_semaphores: Slice<'a, Ref<'a, VkSemaphore>>,
    // Safety: Must be same length as wait_semaphores
    pub wait_stage_masks: Option<Array<'a, PipelineStageFlags>>,
    pub command_buffers: Slice<'a, Mut<'a, VkCommandBuffer>>,
    pub signal_semaphores: Slice<'a, Ref<'a, VkSemaphore>>,
}
structure_type!(SubmitInfoType, 4);

#[repr(C)]
#[derive(Debug, Default)]
#[doc = crate::man_link!(VkMemoryAllocateInfo)]
pub struct MemoryAllocateInfo<Next = Null> {
    pub stype: MemoryAllocateInfoType,
    pub next: Next,
    pub allocation_size: u64,
    pub memory_type_index: u32,
}
structure_type!(MemoryAllocateInfoType, 5);

#[repr(C)]
#[derive(Debug, Default)]
#[doc = crate::man_link!(VkFenceCreateInfo)]
pub struct FenceCreateInfo<Next = Null> {
    pub stype: FenceCreateInfoType,
    pub next: Next,
    pub flags: FenceCreateFlags,
}
structure_type!(FenceCreateInfoType, 8);

#[repr(C)]
#[derive(Debug, Default)]
#[doc = crate::man_link!(VkSemaphoreCreateInfo)]
pub struct SemaphoreCreateInfo<Next = Null> {
    pub stype: SemaphoreCreateInfoType,
    pub next: Next,
    pub flags: SemaphoreCreateFlags,
}
structure_type!(SemaphoreCreateInfoType, 9);

#[repr(C)]
#[derive(Debug, Default)]
#[doc = crate::man_link!(VkBufferCreateInfo)]
pub struct BufferCreateInfo<'a, Next = Null> {
    pub stype: BufferCreateInfoType,
    pub next: Next,
    pub flags: BufferCreateFlags,
    pub size: u64,
    pub usage: BufferUsageFlags,
    pub sharing_mode: SharingMode,
    pub queue_family_indices: Slice<'a, u32>,
}
structure_type!(BufferCreateInfoType, 12);

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkImageCreateInfo)]
pub struct ImageCreateInfo<'a, Next = Null> {
    pub stype: ImageCreateInfoType,
    pub next: Next,
    pub flags: ImageCreateFlags,
    pub image_type: ImageType,
    pub format: Format,
    pub extent: Extent3D,
    pub mip_levels: u32,
    pub array_layers: u32,
    pub samples: SampleCount,
    pub tiling: ImageTiling,
    pub usage: ImageUsageFlags,
    pub sharing_mode: SharingMode,
    pub queue_family_indices: Slice<'a, u32>,
    pub initial_layout: ImageLayout,
}
structure_type!(ImageCreateInfoType, 14);

impl<'a> Default for ImageCreateInfo<'a> {
    fn default() -> Self {
        Self {
            stype: Default::default(),
            next: Default::default(),
            flags: Default::default(),
            image_type: Default::default(),
            format: Default::default(),
            extent: Default::default(),
            mip_levels: 1,
            array_layers: 1,
            samples: Default::default(),
            tiling: Default::default(),
            usage: Default::default(),
            sharing_mode: Default::default(),
            queue_family_indices: Default::default(),
            initial_layout: Default::default(),
        }
    }
}

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkImageViewCreateInfo)]
pub struct VkImageViewCreateInfo<'a, Next = Null> {
    pub stype: ImageViewCreateInfoType,
    pub next: Next,
    pub flags: ImageViewCreateFlags,
    pub image: Ref<'a, VkImage>,
    pub view_type: ImageViewType,
    pub format: Format,
    pub components: ComponentMapping,
    pub subresource_range: ImageSubresourceRange,
}
structure_type!(ImageViewCreateInfoType, 15);

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkShaderModuleCreateInfo)]
pub struct VkShaderModuleCreateInfo<'a, Next = Null> {
    pub stype: ShaderModuleCreateInfoType,
    pub next: Next,
    pub flags: ShaderModuleCreateFlags,
    pub code_size: usize,
    pub code: Array<'a, u32>,
}
structure_type!(ShaderModuleCreateInfoType, 16);

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkPipelineCacheCreateInfo)]
pub struct PipelineCacheCreateInfo<'a, Next = Null> {
    pub stype: ShaderModuleCreateInfoType,
    pub next: Next,
    pub flags: PipelineCacheCreateFlags,
    pub initial_data: Bytes<'a>,
}
structure_type!(PipelineCacheCreateInfoType, 17);

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkSpecializationMapEntry)]
pub struct SpecializationMapEntry {
    pub constant_id: u32,
    pub offset: u32,
    pub size: usize,
}

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkSpecializationInfo)]
pub struct SpecializationInfo<'a> {
    pub map_entries: Slice<'a, SpecializationMapEntry>,
    pub data: Bytes<'a>,
}

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkPipelineShaderStageCreateInfo)]
pub struct PipelineShaderStageCreateInfo<'a, Next = Null> {
    pub stype: PipelineShaderStageCreateInfoType,
    pub next: Next,
    pub flags: PipelineShaderStageCreateFlags,
    pub stage: ShaderStage,
    pub module: Ref<'a, VkShaderModule>,
    pub name: Str<'a>,
    pub specialization_info: Option<&'a SpecializationInfo<'a>>,
}
structure_type!(PipelineShaderStageCreateInfoType, 18);

impl<'a> PipelineShaderStageCreateInfo<'a> {
    const MAIN: Str<'static> = unsafe { Str::new_unchecked(b"main\0") };
    /// Create a vertex shader with entry point "main"
    pub fn vertex(module: &'a crate::shader::ShaderModule) -> Self {
        Self {
            stype: Default::default(),
            next: Default::default(),
            flags: Default::default(),
            stage: ShaderStage::VERTEX,
            module: module.handle(),
            name: Self::MAIN,
            specialization_info: None,
        }
    }
    /// Create a fragment shader with entry point "main"
    pub fn fragment(module: &'a crate::shader::ShaderModule) -> Self {
        Self {
            stype: Default::default(),
            next: Default::default(),
            flags: Default::default(),
            stage: ShaderStage::FRAGMENT,
            module: module.handle(),
            name: Self::MAIN,
            specialization_info: None,
        }
    }
    /// Create a compute shader with entry point "main"
    pub fn compute(module: &'a crate::shader::ShaderModule) -> Self {
        Self {
            stype: Default::default(),
            next: Default::default(),
            flags: Default::default(),
            stage: ShaderStage::COMPUTE,
            module: module.handle(),
            name: Self::MAIN,
            specialization_info: None,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkVertexInputBindingDescription)]
pub struct VertexInputBindingDescription {
    pub binding: u32,
    pub stride: u32,
    pub input_rate: VertexInputRate,
}

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkVertexInputAttributeDescription)]
pub struct VertexInputAttributeDescription {
    pub location: u32,
    pub binding: u32,
    pub format: Format,
    pub offset: u32,
}

#[repr(C)]
#[derive(Debug, Default)]
#[doc = crate::man_link!(VkPipelineVertexInputStateCreateInfo)]
pub struct PipelineVertexInputStateCreateInfo<'a, Next = Null> {
    pub stype: PipelineVertexInputStateCreateInfoType,
    pub next: Next,
    pub flags: PipelineVertexInputStateCreateFlags,
    pub vertex_binding_descriptions: Slice_<'a, VertexInputBindingDescription>,
    pub vertex_attribute_descriptions:
        Slice<'a, VertexInputAttributeDescription>,
}
structure_type!(PipelineVertexInputStateCreateInfoType, 19);

#[repr(C)]
#[derive(Debug, Default)]
#[doc = crate::man_link!(VkPipelineInputAssemblyStateCreateInfo)]
pub struct PipelineInputAssemblyStateCreateInfo<Next = Null> {
    pub stype: PipelineInputAssemblyStateCreateInfoType,
    pub next: Next,
    pub flags: PipelineInputAssemblyStateCreateFlags,
    pub topology: PrimitiveTopology,
    pub primitive_restart_enable: Bool,
}
structure_type!(PipelineInputAssemblyStateCreateInfoType, 20);

#[repr(C)]
#[derive(Debug, Default)]
#[doc = crate::man_link!(VkPipelineTessellationStateCreateInfo)]
pub struct PipelineTessellationStateCreateInfo<Next = Null> {
    pub stype: PipelineTesselationStateCreateInfoType,
    pub next: Next,
    pub flags: PipelineTesselationStateCreateFlags,
    pub patch_control_points: u32,
}
structure_type!(PipelineTesselationStateCreateInfoType, 21);

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkPipelineViewportStateCreateInfo)]
pub struct PipelineViewportStateCreateInfo<'a, Next = Null> {
    pub stype: PipelineViewportStateCreateInfoType,
    pub next: Next,
    pub flags: PipelineViewportStateCreateFlags,
    pub viewports: Slice_<'a, Viewport>,
    pub scissors: Slice<'a, Rect2D>,
}
structure_type!(PipelineViewportStateCreateInfoType, 22);

impl<'a> Default for PipelineViewportStateCreateInfo<'a> {
    /// Viewport state appropriate for dynamic viewport and scissor
    fn default() -> Self {
        const DYN_VIEWPORT: Viewport = Viewport {
            width: 0.,
            height: 0.,
            x: 0.,
            y: 0.,
            min_depth: 0.,
            max_depth: 0.,
        };
        const DYN_SCISSOR: Rect2D = Rect2D {
            offset: Offset2D { x: 0, y: 0 },
            extent: Extent2D { width: 0, height: 0 },
        };
        Self {
            stype: Default::default(),
            next: Default::default(),
            flags: Default::default(),
            viewports: slice(&[DYN_VIEWPORT]),
            scissors: slice(&[DYN_SCISSOR]),
        }
    }
}

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkPipelineRasterizationStateCreateInfo)]
pub struct PipelineRasterizationStateCreateInfo<Next = Null> {
    pub stype: PipelineRasterizationStateCreateInfoType,
    pub next: Next,
    pub flags: PipelineRasterizationStateCreateFlags,
    pub depth_clamp_enable: Bool,
    pub rasterizer_discard_enable: Bool,
    pub polygon_mode: PolygonMode,
    pub cull_mode: CullModeFlags,
    pub front_face: FrontFace,
    pub depth_bias_enable: Bool,
    pub depth_bias_constant_factor: f32,
    pub depth_bias_clamp: f32,
    pub depth_bias_slope_factor: f32,
    pub line_width: f32,
}
structure_type!(PipelineRasterizationStateCreateInfoType, 23);

impl Default for PipelineRasterizationStateCreateInfo {
    fn default() -> Self {
        PipelineRasterizationStateCreateInfo {
            stype: Default::default(),
            next: Default::default(),
            flags: Default::default(),
            depth_clamp_enable: Bool::False,
            rasterizer_discard_enable: Bool::False,
            polygon_mode: PolygonMode::FILL,
            cull_mode: CullModeFlags::empty(),
            front_face: FrontFace::COUNTER_CLOCKWISE,
            depth_bias_enable: Bool::False,
            depth_bias_constant_factor: 0.0,
            depth_bias_clamp: 0.0,
            depth_bias_slope_factor: 0.0,
            line_width: 1.0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Default)]
#[doc = crate::man_link!(VkPipelineMultisampleStateCreateInfo)]
pub struct PipelineMultisampleStateCreateInfo<'a, Next = Null> {
    pub stype: PipelineMultisampleStateCreateInfoType,
    pub next: Next,
    pub flags: PipelineMultisampleStateCreateFlags,
    pub rasterization_samples: SampleCount,
    pub sample_shading_enable: Bool,
    pub min_sample_shading: f32,
    pub sample_mask: Option<&'a u64>,
    pub alpha_to_coverage_enable: Bool,
    pub alpha_to_one_enable: Bool,
}
structure_type!(PipelineMultisampleStateCreateInfoType, 24);

#[repr(C)]
#[derive(Debug, Default)]
#[doc = crate::man_link!(VkStencilOpState)]
pub struct StencilOpState {
    pub fail_op: StencilOp,
    pub pass_op: StencilOp,
    pub depth_fail_op: StencilOp,
    pub compare_op: CompareOp,
    pub compare_mask: u32,
    pub write_mask: u32,
    pub reference: u32,
}

#[repr(C)]
#[derive(Debug, Default)]
#[doc = crate::man_link!(VkPipelineDepthStencilStateCreateInfo)]
pub struct PipelineDepthStencilStateCreateInfo<Next = Null> {
    pub stype: PipelineDepthStencilStateCreateInfoType,
    pub next: Next,
    pub flags: PipelineDepthStencilStateCreateFlags,
    pub depth_test_enable: Bool,
    pub depth_write_enable: Bool,
    pub depth_compare_op: CompareOp,
    pub depth_bounds_test_enable: Bool,
    pub stencil_test_enable: Bool,
    pub front: StencilOpState,
    pub back: StencilOpState,
    pub min_depth_bounds: f32,
    pub max_depth_bounds: f32,
}
structure_type!(PipelineDepthStencilStateCreateInfoType, 25);

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkPipelineColorBlendAttachmentState)]
pub struct PipelineColorBlendAttachmentState {
    pub blend_enable: Bool,
    pub src_color_blend_factor: BlendFactor,
    pub dst_color_blend_factor: BlendFactor,
    pub color_blend_op: BlendOp,
    pub src_alpha_blend_factor: BlendFactor,
    pub dst_alpha_blend_factor: BlendFactor,
    pub alpha_blend_op: BlendOp,
    pub color_write_mask: ColorComponentFlags,
}

const DEFAULT_BLEND: PipelineColorBlendAttachmentState =
    PipelineColorBlendAttachmentState {
        blend_enable: Bool::False,
        src_color_blend_factor: BlendFactor::ONE,
        dst_color_blend_factor: BlendFactor::ONE_MINUS_SRC_ALPHA,
        color_blend_op: BlendOp::ADD,
        src_alpha_blend_factor: BlendFactor::ONE,
        dst_alpha_blend_factor: BlendFactor::ONE_MINUS_SRC_ALPHA,
        alpha_blend_op: BlendOp::ADD,
        color_write_mask: ColorComponentFlags::RGBA,
    };

impl Default for PipelineColorBlendAttachmentState {
    /// Blending disabled, and premultiplied alpha blending parameters.
    fn default() -> Self {
        DEFAULT_BLEND
    }
}

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkPipelineColorBlendStateCreateInfo)]
pub struct PipelineColorBlendStateCreateInfo<'a, Next = Null> {
    pub stype: PipelineColorBlendStateCreateInfoType,
    pub next: Next,
    pub flags: PipelineColorBlendStateCreateFlags,
    pub logic_op_enable: Bool,
    pub logic_op: LogicOp,
    pub attachments: Slice_<'a, PipelineColorBlendAttachmentState>,
    pub blend_constants: [f32; 4],
}
structure_type!(PipelineColorBlendStateCreateInfoType, 26);

impl Default for PipelineColorBlendStateCreateInfo<'static> {
    /// One color attachment with blending disabled
    fn default() -> Self {
        Self {
            stype: Default::default(),
            next: Default::default(),
            flags: Default::default(),
            logic_op_enable: Bool::False,
            logic_op: LogicOp::CLEAR,
            attachments: std::slice::from_ref(&DEFAULT_BLEND).into(),
            blend_constants: Default::default(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Default)]
#[doc = crate::man_link!(VkPipelineDynamicStateCreateInfo)]
pub struct PipelineDynamicStateCreateInfo<'a, Next = Null> {
    pub stype: PipelineDynamicStateCreateInfoType,
    pub next: Next,
    pub flags: DynamicStateCreateFlags,
    pub dynamic_states: Slice_<'a, DynamicState>,
}
structure_type!(PipelineDynamicStateCreateInfoType, 27);

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkGraphicsPipelineCreateInfo)]
pub struct VkGraphicsPipelineCreateInfo<'a, Next = Null> {
    pub stype: GraphicsPipelineCreateInfoType,
    pub next: Next,
    pub flags: PipelineCreateFlags,
    pub stages: Slice_<'a, PipelineShaderStageCreateInfo<'a>>,
    pub vertex_input_state: &'a PipelineVertexInputStateCreateInfo<'a>,
    pub input_assembly_state: &'a PipelineInputAssemblyStateCreateInfo,
    pub tessellation_state: Option<&'a PipelineTessellationStateCreateInfo>,
    pub viewport_state: &'a PipelineViewportStateCreateInfo<'a>,
    pub rasterization_state: &'a PipelineRasterizationStateCreateInfo,
    pub multisample_state: &'a PipelineMultisampleStateCreateInfo<'a>,
    pub depth_stencil_state: Option<&'a PipelineDepthStencilStateCreateInfo>,
    pub color_blend_state: &'a PipelineColorBlendStateCreateInfo<'a>,
    pub dynamic_state: Option<&'a PipelineDynamicStateCreateInfo<'a>>,
    pub layout: Ref<'a, VkPipelineLayout>,
    pub render_pass: Ref<'a, VkRenderPass>,
    pub subpass: u32,
    pub base_pipeline_handle: Option<Ref<'a, VkPipeline>>,
    pub base_pipeline_index: u32,
}
structure_type!(GraphicsPipelineCreateInfoType, 28);

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkComputePipelineCreateInfo)]
pub struct ComputePipelineCreateInfo<'a, Next = Null> {
    pub stype: ComputePipelineCreateInfoType,
    pub next: Next,
    pub flags: PipelineCreateFlags,
    pub stage: PipelineShaderStageCreateInfo<'a>,
    pub layout: Ref<'a, VkPipelineLayout>,
    pub base_pipeline_handle: Option<Ref<'a, VkPipeline>>,
    pub base_pipeline_index: u32,
}
structure_type!(ComputePipelineCreateInfoType, 29);

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkPushConstantRange)]
pub struct PushConstantRange {
    pub stage_flags: ShaderStageFlags,
    pub offset: u32,
    pub size: u32,
}

#[repr(C)]
#[derive(Debug, Default)]
#[doc = crate::man_link!(VkPipelineLayoutCreateInfo)]
pub struct PipelineLayoutCreateInfo<'a, Next = Null> {
    pub stype: PipelineLayoutCreateInfoType,
    pub next: Next,
    pub flags: PipelineLayoutCreateFlags,
    pub set_layouts: Slice_<'a, Ref<'a, VkDescriptorSetLayout>>,
    pub push_constant_ranges: Slice<'a, PushConstantRange>,
}
structure_type!(PipelineLayoutCreateInfoType, 30);

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkSamplerCreateInfo)]
pub struct SamplerCreateInfo<Next = Null> {
    pub stype: SamplerCreateInfoType,
    pub next: Next,
    pub flags: SamplerCreateFlags,
    pub mag_filter: Filter,
    pub min_filter: Filter,
    pub mipmap_mode: SamplerMipmapMode,
    pub address_mode_u: SamplerAddressMode,
    pub address_mode_v: SamplerAddressMode,
    pub address_mode_w: SamplerAddressMode,
    pub mip_lod_bias: f32,
    pub anisotropy_enable: Bool,
    pub max_anisotropy: f32,
    pub compare_enable: Bool,
    pub compare_op: CompareOp,
    pub min_lod: f32,
    pub max_lod: f32,
    pub border_color: BorderColor,
    pub unnormalized_coordinates: Bool,
}
structure_type!(SamplerCreateInfoType, 31);

impl Default for SamplerCreateInfo {
    fn default() -> Self {
        Self {
            stype: Default::default(),
            next: Default::default(),
            flags: Default::default(),
            mag_filter: Default::default(),
            min_filter: Default::default(),
            mipmap_mode: Default::default(),
            address_mode_u: Default::default(),
            address_mode_v: Default::default(),
            address_mode_w: Default::default(),
            mip_lod_bias: Default::default(),
            anisotropy_enable: Default::default(),
            max_anisotropy: Default::default(),
            compare_enable: Default::default(),
            compare_op: Default::default(),
            min_lod: Default::default(),
            max_lod: 1000.0,
            border_color: Default::default(),
            unnormalized_coordinates: Default::default(),
        }
    }
}

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkDescriptorSetLayoutBinding)]
pub struct VkDescriptorSetLayoutBinding<'a> {
    pub binding: u32,
    pub descriptor_type: DescriptorType,
    pub descriptor_count: u32,
    pub stage_flags: ShaderStageFlags,
    // Safety: Must be descriptor_count long
    pub immutable_samplers: Option<Array<'a, Ref<'a, VkSampler>>>,
}

#[repr(C)]
#[derive(Debug, Default)]
#[doc = crate::man_link!(VkDescriptorSetLayoutCreateInfo)]
pub struct VkDescriptorSetLayoutCreateInfo<'a, Next = Null> {
    pub stype: DescriptorSetLayoutCreateInfoType,
    pub next: Next,
    pub flags: DescriptorSetLayoutCreateFlags,
    pub bindings: Slice_<'a, VkDescriptorSetLayoutBinding<'a>>,
}
structure_type!(DescriptorSetLayoutCreateInfoType, 32);

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkDescriptorPoolSize)]
pub struct DescriptorPoolSize {
    pub descriptor_type: DescriptorType,
    pub descriptor_count: u32,
}

#[repr(C)]
#[derive(Debug, Default)]
#[doc = crate::man_link!(VkDescriptorPoolCreateInfo)]
pub struct DescriptorPoolCreateInfo<'a, Next = Null> {
    pub stype: DescriptorPoolCreateInfoType,
    pub next: Next,
    pub flags: DescriptorPoolCreateFlags,
    pub max_sets: u32,
    pub pool_sizes: Slice<'a, DescriptorPoolSize>,
}
structure_type!(DescriptorPoolCreateInfoType, 33);

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkDescriptorSetAllocateInfo)]
pub struct DescriptorSetAllocateInfo<'a, Next = Null> {
    pub stype: DescriptorSetAllocateInfoType,
    pub next: Next,
    pub descriptor_pool: Mut<'a, VkDescriptorPool>,
    pub set_layouts: Slice<'a, Ref<'a, VkDescriptorSetLayout>>,
}
structure_type!(DescriptorSetAllocateInfoType, 34);

#[repr(C)]
#[derive(Debug, Default)]
#[doc = crate::man_link!(VkDescriptorImageInfo)]
pub struct VkDescriptorImageInfo<'a> {
    pub sampler: Option<Ref<'a, VkSampler>>,
    pub image_view: Option<Ref<'a, VkImageView>>,
    pub image_layout: ImageLayout,
}

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkDescriptorBufferInfo)]
pub struct VkDescriptorBufferInfo<'a> {
    pub buffer: Ref<'a, VkBuffer>,
    pub offset: u64,
    pub range: u64,
}

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkWriteDescriptorSet)]
pub struct VkWriteDescriptorSet<'a, Next = Null> {
    pub stype: WriteDescriptorSetType,
    pub next: Next,
    pub dst_set: Mut<'a, VkDescriptorSet>,
    pub dst_binding: u32,
    pub dst_array_element: u32,
    pub descriptor_count: u32,
    pub descriptor_type: DescriptorType,
    pub image_info: Option<Array<'a, VkDescriptorImageInfo<'a>>>,
    pub buffer_info: Option<Array<'a, VkDescriptorBufferInfo<'a>>>,
    pub texel_buffer_view: Option<Array<'a, Ref<'a, VkBufferView>>>,
}
structure_type!(WriteDescriptorSetType, 35);

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkCopyDescriptorSet)]
pub struct VkCopyDescriptorSet<'a, Next = Null> {
    pub stype: WriteDescriptorSetType,
    pub next: Next,
    pub src_set: Ref<'a, VkDescriptorSet>,
    pub src_binding: u32,
    pub src_array_element: u32,
    pub dst_set: Mut<'a, VkDescriptorSet>,
    pub dst_binding: u32,
    pub dst_array_element: u32,
    pub descriptor_count: u32,
}
structure_type!(CopyDescriptorSetType, 35);

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkFramebufferCreateInfo)]
pub struct VkFramebufferCreateInfo<'a, Next = Null> {
    pub stype: FramebufferCreateInfoType,
    pub next: Next,
    pub flags: FramebufferCreateFlags,
    pub render_pass: Ref<'a, VkRenderPass>,
    pub attachments: Slice<'a, Ref<'a, VkImageView>>,
    pub width: u32,
    pub height: u32,
    pub layers: u32,
}
structure_type!(FramebufferCreateInfoType, 37);

#[repr(C)]
#[derive(Debug, Default)]
#[doc = crate::man_link!(VkAttachmentDescription)]
pub struct AttachmentDescription {
    pub flags: AttachmentDescriptionFlags,
    pub format: Format,
    pub samples: SampleCount,
    pub load_op: AttachmentLoadOp,
    pub store_op: AttachmentStoreOp,
    pub stencil_load_op: AttachmentLoadOp,
    pub stencil_store_op: AttachmentStoreOp,
    pub initial_layout: ImageLayout,
    pub final_layout: ImageLayout,
}

#[repr(C)]
#[derive(Debug, Default)]
#[doc = crate::man_link!(VkAttachmentReference)]
pub struct AttachmentReference {
    /// Either an index in the attachments member of [`RenderPassCreateInfo`] or
    /// u32::MAX if unused
    pub attachment: u32,
    pub layout: ImageLayout,
}

#[repr(C)]
#[derive(Debug)]
/// Create with
/// [`vk::SubpassDescription::try_into`](SubpassDescription).
#[doc = crate::man_link!(VkSubpassDescription)]
pub struct VkSubpassDescription<'a> {
    flags: SubpassDescriptionFlags,
    pipeline_bind_point: PipelineBindPoint,
    pub(crate) input_attachments: Slice<'a, AttachmentReference>,
    pub(crate) color_attachments: Slice<'a, AttachmentReference>,
    /// Safety: Must be the same length as color_attachments
    pub(crate) resolve_attachments: Option<Array<'a, AttachmentReference>>,
    pub(crate) depth_stencil_attachments:
        Option<Array<'a, AttachmentReference>>,
    pub(crate) preserve_attachments: Slice<'a, AttachmentReference>,
}

impl<'a> Default for VkSubpassDescription<'a> {
    fn default() -> Self {
        SubpassDescription::default().try_into().unwrap()
    }
}

#[derive(Default)]
#[doc = crate::man_link!(VkSubpassDescription)]
pub struct SubpassDescription<'a> {
    pub input_attachments: &'a [AttachmentReference],
    pub color_attachments: &'a [AttachmentReference],
    /// Must be either empty or the same length as color_attachments
    pub resolve_attachments: &'a [AttachmentReference],
    /// Must be either empty or the same length as color_attachments
    pub depth_stencil_attachments: &'a [AttachmentReference],
    pub preserve_attachments: &'a [AttachmentReference],
}
impl<'a> From<SubpassDescription<'a>> for VkSubpassDescription<'a> {
    #[inline]
    fn from(value: SubpassDescription<'a>) -> Self {
        if !value.resolve_attachments.is_empty()
            && value.resolve_attachments.len() != value.color_attachments.len()
        {
            panic!("Different number of resolve and color attachments")
        }
        if !value.depth_stencil_attachments.is_empty()
            && value.depth_stencil_attachments.len()
                != value.color_attachments.len()
        {
            panic!("Different number of depth-stencil and color attachments")
        }
        Self {
            flags: Default::default(),
            pipeline_bind_point: PipelineBindPoint::GRAPHICS,
            input_attachments: value.input_attachments.into(),
            color_attachments: value.color_attachments.into(),
            resolve_attachments: Array::from_slice(value.resolve_attachments),
            depth_stencil_attachments: Array::from_slice(
                value.depth_stencil_attachments,
            ),
            preserve_attachments: value.preserve_attachments.into(),
        }
    }
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone)]
#[doc = crate::man_link!(VkSubpassDependency)]
pub struct SubpassDependency {
    pub src_subpass: u32,
    pub dst_subpass: u32,
    pub src_stage_mask: PipelineStageFlags,
    pub dst_stage_mask: PipelineStageFlags,
    pub src_access_mask: AccessFlags,
    pub dst_access_mask: AccessFlags,
    pub dependency_flags: DependencyFlags,
}

#[repr(C)]
#[derive(Debug, Default)]
#[doc = crate::man_link!(VkRenderPassCreateInfo)]
pub struct RenderPassCreateInfo<'a, Next = Null> {
    pub stype: RenderPassCreateInfoType,
    pub next: Next,
    pub flags: RenderPassCreateFlags,
    pub attachments: Slice_<'a, AttachmentDescription>,
    pub subpasses: Slice<'a, VkSubpassDescription<'a>>,
    pub dependencies: Slice<'a, SubpassDependency>,
}
structure_type!(RenderPassCreateInfoType, 38);

#[repr(C)]
#[derive(Debug, Default)]
#[doc = crate::man_link!(VkCommandPoolCreateInfo)]
pub struct CommandPoolCreateInfo<Next = Null> {
    pub stype: CommandPoolCreateInfoType,
    pub next: Next,
    pub flags: CommandPoolCreateFlags,
    pub queue_family_index: u32,
}
structure_type!(CommandPoolCreateInfoType, 39);

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkCommandBufferAllocateInfo)]
pub struct CommandBufferAllocateInfo<'a, Next = Null> {
    pub stype: CommandBufferAllocateInfoType,
    pub next: Next,
    pub pool: Mut<'a, VkCommandPool>,
    pub level: CommandBufferLevel,
    pub count: u32,
}
structure_type!(CommandBufferAllocateInfoType, 40);

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkCommandBufferInheritanceInfo)]
pub struct CommandBufferInheritanceInfo<'a, Next = Null> {
    pub stype: CommandBufferInheritanceInfoType,
    pub next: Next,
    pub render_pass: Ref<'a, VkRenderPass>,
    pub subpass: u32,
    pub framebuffer: Option<Ref<'a, VkFramebuffer>>,
    pub occlusion_query_enable: Bool,
    pub query_flags: QueryControlFlags,
    pub pipeline_statistics: QueryPipelineStatisticFlags,
}
structure_type!(CommandBufferInheritanceInfoType, 41);

#[repr(C)]
#[derive(Debug, Default)]
#[doc = crate::man_link!(VkCommandBufferBeginInfo)]
pub struct CommandBufferBeginInfo<'a, Next = Null> {
    pub stype: CommandBufferBeginInfoType,
    pub next: Next,
    pub flags: CommandBufferUsageFlags,
    pub inheritance_info: Option<&'a CommandBufferInheritanceInfo<'a>>,
}
structure_type!(CommandBufferBeginInfoType, 42);

#[repr(C)]
#[doc = crate::man_link!(VkRenderPassBeginInfo)]
pub struct RenderPassBeginInfo<'a, Next = Null> {
    pub stype: RenderPassBeginInfoType,
    pub next: Next,
    pub render_pass: Ref<'a, VkRenderPass>,
    pub framebuffer: Ref<'a, VkFramebuffer>,
    pub render_area: Rect2D,
    pub clear_values: Slice<'a, ClearValue>,
}
structure_type!(RenderPassBeginInfoType, 43);

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkBufferMemoryBarrier)]
pub struct BufferMemoryBarrier<'a, Next = Null> {
    pub stype: BufferMemoryBarrierType,
    pub next: Next,
    pub src_access_mask: AccessFlags,
    pub dst_access_mask: AccessFlags,
    pub src_queue_family_index: u32,
    pub dst_queue_family_index: u32,
    pub buffer: Ref<'a, VkBuffer>,
    pub offset: u64,
    pub size: u64,
}
structure_type!(BufferMemoryBarrierType, 44);

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkImageMemoryBarrier)]
pub struct ImageMemoryBarrier<'a, Next = Null> {
    pub stype: ImageMemoryBarrierType,
    pub next: Next,
    pub src_access_mask: AccessFlags,
    pub dst_access_mask: AccessFlags,
    pub old_layout: ImageLayout,
    pub new_layout: ImageLayout,
    pub src_queue_family_index: u32,
    pub dst_queue_family_index: u32,
    pub image: Ref<'a, VkImage>,
    pub subresource_range: ImageSubresourceRange,
}
structure_type!(ImageMemoryBarrierType, 45);

#[repr(C)]
#[derive(Debug, Default)]
#[doc = crate::man_link!(VkMemoryBarrier)]
pub struct MemoryBarrier<Next = Null> {
    pub stype: MemoryBarrierType,
    pub next: Next,
    pub src_access_mask: AccessFlags,
    pub dst_access_mask: AccessFlags,
}
structure_type!(MemoryBarrierType, 46);

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkMetalSurfaceCreateInfoEXT)]
pub struct MetalSurfaceCreateInfoEXT<Next = Null> {
    pub stype: MetalSurfaceCreateInfoEXTType,
    pub next: Next,
    pub flags: MetalSurfaceCreateFlagsEXT,
    pub layer: NonNull<c_void>,
}
structure_type!(MetalSurfaceCreateInfoEXTType, 1000217000);

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkXlibSurfaceCreateInfoKHR)]
pub struct XlibSurfaceCreateInfoKHR<Next = Null> {
    pub stype: XlibSurfaceCreateInfoKHRType,
    pub next: Next,
    pub flags: XlibSurfaceCreateFlagsKHR,
    pub display: NonNull<c_void>,
    pub window: usize,
}
structure_type!(XlibSurfaceCreateInfoKHRType, 1000004000);

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkWaylandSurfaceCreateInfoKHR)]
pub struct WaylandSurfaceCreateInfoKHR<Next = Null> {
    pub stype: WaylandSurfaceCreateInfoKHRType,
    pub next: Next,
    pub flags: WaylandSurfaceCreateFlagsKHR,
    pub display: NonNull<c_void>,
    pub surface: NonNull<c_void>,
}
structure_type!(WaylandSurfaceCreateInfoKHRType, 1000006000);

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkWin32SurfaceCreateInfoKHR)]
pub struct Win32SurfaceCreateInfoKHR<Next = Null> {
    pub stype: Win32SurfaceCreateInfoKHRType,
    pub next: Next,
    pub flags: Win32SurfaceCreateFlagsKHR,
    pub hinstance: NonNull<c_void>,
    pub hwnd: NonNull<c_void>,
}
structure_type!(Win32SurfaceCreateInfoKHRType, 1000009000);

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkSurfaceCapabilitiesKHR)]
pub struct SurfaceCapabilitiesKHR {
    pub min_image_count: u32,
    pub max_image_count: u32,
    pub current_extent: Extent2D,
    pub min_image_extent: Extent2D,
    pub max_image_extent: Extent2D,
    pub max_image_array_layers: u32,
    pub supported_transforms: SurfaceTransformFlagsKHR,
    pub current_transform: SurfaceTransformKHR,
    pub supported_composite_alpha: CompositeAlphaFlagsKHR,
    pub supported_usage_flags: ImageUsageFlags,
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq)]
#[doc = crate::man_link!(VkSurfaceFormatKHR)]
pub struct SurfaceFormatKHR {
    pub format: Format,
    pub color_space: ColorSpaceKHR,
}

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkSwapchainCreateInfoKHR)]
pub struct VkSwapchainCreateInfoKHR<'a, Next = Null> {
    pub stype: SwapchainCreateInfoKHRType,
    pub next: Next,
    pub flags: SwapchainCreateFlagsKHR,
    pub surface: Mut<'a, VkSurfaceKHR>,
    pub min_image_count: u32,
    pub image_format: Format,
    pub image_color_space: ColorSpaceKHR,
    pub image_extent: Extent2D,
    pub image_array_layers: u32,
    pub image_usage: ImageUsageFlags,
    pub image_sharing_mode: SharingMode,
    pub queue_family_indices: Slice<'a, u32>,
    pub pre_transform: SurfaceTransformKHR,
    pub composite_alpha: CompositeAlphaKHR,
    pub present_mode: PresentModeKHR,
    pub clipped: Bool,
    pub old_swapchain: Option<Mut<'a, VkSwapchainKHR>>,
}
structure_type!(SwapchainCreateInfoKHRType, 1000001000);

#[repr(C)]
#[derive(Debug)]
#[doc = crate::man_link!(VkPresentInfoKHR)]
pub struct PresentInfoKHR<'a, Next = Null> {
    pub stype: PresentInfoType,
    pub next: Next,
    pub wait: Slice<'a, Mut<'a, VkSemaphore>>,
    /// Safety: The following members are arrays of the same length
    pub swapchains: Slice<'a, Mut<'a, VkSwapchainKHR>>,
    pub indices: Array<'a, u32>,
    pub results: Option<ArrayMut<'a, VkResult>>,
}
structure_type!(PresentInfoType, 1000001001);
