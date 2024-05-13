// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::types::{Extent2D, Extent3D};

use bitflags::bitflags;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// 32 bit bool
pub enum Bool {
    False = 0,
    True = 1,
}
impl Default for Bool {
    fn default() -> Self {
        Self::False
    }
}
impl From<Bool> for bool {
    #[inline]
    fn from(b: Bool) -> Self {
        b == Bool::True
    }
}
impl From<bool> for Bool {
    #[inline]
    fn from(b: bool) -> Self {
        if b {
            Bool::True
        } else {
            Bool::False
        }
    }
}
impl Bool {
    pub fn as_bool(self) -> bool {
        self == Self::True
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    #[doc = crate::man_link!(VkInstanceCreateFlagBits)]
    pub struct InstanceCreateFlags: u32 {
        const INSTANCE_CREATE_ENUMERATE_PORTABILITY_BIT_KHR = 0x00000001;
    }
}

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq)]
#[doc = crate::man_link!(VkPhysicalDeviceType)]
pub struct PhysicalDeviceType(u32);
impl PhysicalDeviceType {
    pub const OTHER: Self = Self(0);
    pub const INTEGRATED_GPU: Self = Self(1);
    pub const DISCRETE_GPU: Self = Self(2);
    pub const VIRTUAL_GPU: Self = Self(3);
    pub const CPU: Self = Self(4);
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[doc = crate::man_link!(VkSampleCountFlagBits)]
pub struct SampleCount(u32);
impl SampleCount {
    pub const _1: Self = Self(0x01);
    pub const _2: Self = Self(0x02);
    pub const _4: Self = Self(0x04);
    pub const _8: Self = Self(0x08);
    pub const _16: Self = Self(0x10);
    pub const _32: Self = Self(0x20);
    pub const _64: Self = Self(0x40);
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    #[doc = crate::man_link!(VkSampleCountFlagBits)]
    pub struct SampleCountFlags: u32 {
        const _1 = 0x01;
        const _2 = 0x02;
        const _4 = 0x04;
        const _8 = 0x08;
        const _16 = 0x10;
        const _32 = 0x20;
        const _64 = 0x40;
    }
}

impl std::fmt::Debug for SampleCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        SampleCountFlags::from(*self).fmt(f)
    }
}
impl Default for SampleCount {
    fn default() -> Self {
        Self::_1
    }
}
impl From<SampleCount> for SampleCountFlags {
    fn from(bit: SampleCount) -> Self {
        Self::from_bits(bit.0).unwrap()
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    #[doc = crate::man_link!(VkQueueFlagBits)]
    pub struct QueueFlags: u32 {
        const GRAPHICS = 0x01;
        const COMPUTE = 0x02;
        const TRANSFER = 0x04;
        const SPARSE_BINDING = 0x08;
        const PROTECTED = 0x10;
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    /// Reserved
    pub struct DeviceCreateFlags: u32 {}
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    #[doc = crate::man_link!(VkDeviceQueueCreateFlagBits)]
    pub struct DeviceQueueCreateFlags: u32 {
        const PROTECTED = 0x1;
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    #[doc = crate::man_link!(VkPipelineStageFlagBits)]
    pub struct PipelineStageFlags: u32 {
        const TOP_OF_PIPE = 0x00000001;
        const DRAW_INDIRECT = 0x00000002;
        const VERTEX_INPUT = 0x00000004;
        const VERTEX_SHADER = 0x00000008;
        const TESSELLATION_CONTROL_SHADER = 0x00000010;
        const TESSELLATION_EVALUATION_SHADER = 0x00000020;
        const GEOMETRY_SHADER = 0x00000040;
        const FRAGMENT_SHADER = 0x00000080;
        const EARLY_FRAGMENT_TESTS = 0x00000100;
        const LATE_FRAGMENT_TESTS = 0x00000200;
        const COLOR_ATTACHMENT_OUTPUT = 0x00000400;
        const COMPUTE_SHADER = 0x00000800;
        const TRANSFER = 0x00001000;
        const BOTTOM_OF_PIPE = 0x00002000;
        const HOST = 0x00004000;
        const ALL_GRAPHICS = 0x00008000;
        const ALL_COMMANDS = 0x00010000;
        const TRANSFORM_FEEDBACK_EXT = 0x01000000;
        const CONDITIONAL_RENDERING_EXT = 0x00040000;
        const ACCELERATION_STRUCTURE_BUILD_KHR = 0x02000000;
        const RAY_TRACING_SHADER_KHR = 0x00200000;
        const FRAGMENT_DENSITY_PROCESS_EXT = 0x00800000;
        const FRAGMENT_SHADING_RATE_ATTACHMENT_KHR = 0x00400000;
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[doc = crate::man_link!(VkShaderStage)]
pub struct ShaderStage(u32);
impl ShaderStage {
    pub const VERTEX: Self = Self(0x01);
    pub const TESSELLATION_CONTROL: Self = Self(0x02);
    pub const TESSELLATION_EVALUATION: Self = Self(0x04);
    pub const GEOMETRY: Self = Self(0x08);
    pub const FRAGMENT: Self = Self(0x10);
    pub const COMPUTE: Self = Self(0x20);
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    #[doc = crate::man_link!(VkDependencyFlagBits)]
    pub struct DependencyFlags: u32 {
        const BY_REGION = 0x1;
        const DEVICE_GROUP = 0x4;
        const VIEW_LOCAL = 0x2;
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    #[doc = crate::man_link!(VkAccessFlagBits)]
    pub struct AccessFlags: u32 {
        const INDIRECT_COMMAND_READ = 0x00001;
        const INDEX_READ = 0x00002;
        const VERTEX_ATTRIBUTE_READ = 0x00004;
        const UNIFORM_READ = 0x00008;
        const INPUT_ATTACHMENT_READ = 0x00010;
        const SHADER_READ = 0x00020;
        const SHADER_WRITE = 0x00040;
        const COLOR_ATTACHMENT_READ = 0x00080;
        const COLOR_ATTACHMENT_WRITE = 0x00100;
        const DEPTH_STENCIL_ATTACHMENT_READ = 0x00200;
        const DEPTH_STENCIL_ATTACHMENT_WRITE = 0x00400;
        const TRANSFER_READ = 0x00800;
        const TRANSFER_WRITE = 0x01000;
        const HOST_READ = 0x02000;
        const HOST_WRITE = 0x04000;
        const MEMORY_READ = 0x08000;
        const MEMORY_WRITE = 0x10000;
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[doc = crate::man_link!(VkSubpassContents)]
pub struct SubpassContents(u32);
impl SubpassContents {
    pub const INLINE: Self = Self(0);
    pub const SECONDARY_COMMAND_BUFFERS: Self = Self(1);
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    #[doc = crate::man_link!(VkAttachmentDescriptionFlagBits)]
    pub struct AttachmentDescriptionFlags: u32 {
        const MAY_ALIAS = 0x1;
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    #[doc = crate::man_link!(VkFenceCreateFlagBits)]
    pub struct FenceCreateFlags: u32 {
        const SIGNALLED = 0x1;
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    /// Reserved
    pub struct SemaphoreCreateFlags: u32 {}
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    #[doc = crate::man_link!(VkBufferCreateFlagBits)]
    pub struct BufferCreateFlags: u32 {
        const SPARSE_BINDING = 0x1;
        const SPARSE_RESIDENCY = 0x2;
        const SPARSE_ALIASED = 0x4;
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    #[doc = crate::man_link!(VkBufferUsageFlagBits)]
    pub struct BufferUsageFlags: u32 {
        const TRANSFER_SRC = 0x00001;
        const TRANSFER_DST = 0x00002;
        const UNIFORM_TEXEL_BUFFER = 0x00004;
        const STORAGE_TEXEL_BUFFER = 0x00008;
        const UNIFORM_BUFFER = 0x00010;
        const STORAGE_BUFFER = 0x00020;
        const INDEX_BUFFER = 0x00040;
        const VERTEX_BUFFER = 0x00080;
        const INDIRECT_BUFFER = 0x00100;
        const SHADER_DEVICE_ADDRESS = 0x20000;
    }
}

impl BufferUsageFlags {
    /// Does the usage support arbitrary indexing?
    pub fn indexable(self) -> bool {
        let allowed =
            Self::TRANSFER_SRC | Self::TRANSFER_DST | Self::INDIRECT_BUFFER;
        !allowed.contains(self)
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    #[doc = crate::man_link!(VkImageCreateFlagBits)]
    pub struct ImageCreateFlags: u32 {
        const SPARSE_BINDING = 0x001;
        const SPARSE_RESIDENCY = 0x002;
        const SPARSE_ALIASED = 0x004;
        const MUTABLE_FORMAT = 0x008;
        const CUBE_COMPATIBLE = 0x010;
        const ALIAS = 0x400;
        const SPLIT_INSTANCE_BIND_REGIONS = 0x040;
        const _2D_ARRAY_COMPATIBLE = 0x020;
        const BLOCK_TEXEL_VIEW_COMPATIBLE = 0x080;
        const EXTENDED_USAGE = 0x100;
        const PROTECTED = 0x800;
        const DISJOINT = 0x200;
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[doc = crate::man_link!(VkImageType)]
pub struct ImageType(u32);
impl ImageType {
    pub const _1D: Self = Self(0);
    pub const _2D: Self = Self(1);
    pub const _3D: Self = Self(2);
}
impl Default for ImageType {
    fn default() -> Self {
        Self::_2D
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[doc = crate::man_link!(VkImageTiling)]
pub struct ImageTiling(u32);
impl ImageTiling {
    pub const OPTIMAL: Self = Self(0);
    pub const LINEAR: Self = Self(1);
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    /// Reserved
    pub struct ImageViewCreateFlags: u32 {}
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ImageViewType(u32);
impl ImageViewType {
    pub const _1D: Self = Self(0);
    pub const _2D: Self = Self(1);
    pub const _3D: Self = Self(2);
    pub const CUBE: Self = Self(3);
    pub const _1D_ARRAY: Self = Self(4);
    pub const _2D_ARRAY: Self = Self(5);
    pub const CUBE_ARRAY: Self = Self(6);
}
impl Default for ImageViewType {
    fn default() -> Self {
        Self::_2D
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[doc = crate::man_link!(VkComponentSwizzle)]
pub struct ComponentSwizzle(u32);
impl ComponentSwizzle {
    pub const IDENTITY: Self = Self(0);
    pub const ZERO: Self = Self(1);
    pub const ONE: Self = Self(2);
    pub const R: Self = Self(3);
    pub const G: Self = Self(4);
    pub const B: Self = Self(5);
    pub const A: Self = Self(6);
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    /// Reserved
    pub struct MetalSurfaceCreateFlagsEXT: u32 {}
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    /// Reserved
    pub struct XlibSurfaceCreateFlagsKHR: u32 {}
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    /// Reserved
    pub struct WaylandSurfaceCreateFlagsKHR: u32 {}
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    /// Reserved
    pub struct Win32SurfaceCreateFlagsKHR: u32 {}
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[doc = crate::man_link!(VkSurfaceTransformKHR)]
pub struct SurfaceTransformKHR(u32);
impl SurfaceTransformKHR {
    pub const IDENTITY: Self = Self(0x01);
    pub const ROTATE_90: Self = Self(0x002);
    pub const ROTATE_180: Self = Self(0x004);
    pub const ROTATE_270: Self = Self(0x008);
    pub const HORIZONTAL_MIRROR: Self = Self(0x010);
    pub const HORIZONTAL_MIRROR_ROTATE_90: Self = Self(0x020);
    pub const HORIZONTAL_MIRROR_ROTATE_180: Self = Self(0x040);
    pub const HORIZONTAL_MIRROR_ROTATE_270: Self = Self(0x080);
    pub const INHERIT: Self = Self(0x100);
}
impl std::fmt::Debug for SurfaceTransformKHR {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        SurfaceTransformFlagsKHR::from_bits_truncate(self.0).fmt(f)
    }
}
impl Default for SurfaceTransformKHR {
    fn default() -> Self {
        Self::IDENTITY
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    #[doc = crate::man_link!(VkSurfaceTransformFlagBits)]
    pub struct SurfaceTransformFlagsKHR: u32 {
        const IDENTITY = 0x01;
        const ROTATE_90 = 0x002;
        const ROTATE_180 = 0x004;
        const ROTATE_270 = 0x008;
        const HORIZONTAL_MIRROR = 0x010;
        const HORIZONTAL_MIRROR_ROTATE_90 = 0x020;
        const HORIZONTAL_MIRROR_ROTATE_180 = 0x040;
        const HORIZONTAL_MIRROR_ROTATE_270 = 0x080;
        const INHERIT = 0x100;
    }
}
impl From<SurfaceTransformKHR> for SurfaceTransformFlagsKHR {
    fn from(bit: SurfaceTransformKHR) -> Self {
        Self::from_bits(bit.0).unwrap()
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[doc = crate::man_link!(VkCompositeAlphaKHR)]
pub struct CompositeAlphaKHR(u32);
impl CompositeAlphaKHR {
    pub const OPAQUE: Self = Self(0x1);
    pub const PRE_MULTIPLIED: Self = Self(0x2);
    pub const POST_MULTIPLIED: Self = Self(0x4);
    pub const INHERIT: Self = Self(0x8);
}
impl std::fmt::Debug for CompositeAlphaKHR {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        CompositeAlphaFlagsKHR::from_bits_truncate(self.0).fmt(f)
    }
}
impl Default for CompositeAlphaKHR {
    fn default() -> Self {
        Self::OPAQUE
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    #[doc = crate::man_link!(VkCompositeAlphaFlagBits)]
    pub struct CompositeAlphaFlagsKHR: u32 {
        const OPAQUE = 0x1;
        const PRE_MULTIPLIED = 0x2;
        const POST_MULTIPLIED = 0x4;
        const INHERIT = 0x8;
    }
}
impl From<CompositeAlphaKHR> for CompositeAlphaFlagsKHR {
    fn from(bit: CompositeAlphaKHR) -> Self {
        Self::from_bits(bit.0).unwrap()
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    #[doc = crate::man_link!(VkImageUsageFlagBits)]
    pub struct ImageUsageFlags: u32 {
        const TRANSFER_SRC = 0x01;
        const TRANSFER_DST = 0x02;
        const SAMPLED = 0x04;
        const STORAGE = 0x08;
        const COLOR_ATTACHMENT = 0x10;
        const DEPTH_STENCIL_ATTACHMENT = 0x20;
        const TRANSIENT_ATTACHMENT = 0x40;
        const INPUT_ATTACHMENT = 0x80;
    }
}

impl ImageUsageFlags {
    /// Does the usage support arbitrary shader access?
    pub fn indexable(self) -> bool {
        self.intersects(Self::STORAGE)
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[doc = crate::man_link!(VkIndexType)]
pub struct IndexType(u32);
impl IndexType {
    pub const UINT16: Self = Self(0);
    pub const UINT32: Self = Self(1);
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[doc = crate::man_link!(VkFormat)]
pub struct Format(u32);
impl Format {
    pub const UNDEFINED: Self = Self(0);
    pub const R4G4_UNORM_PACK8: Self = Self(1);
    pub const R4G4B4A4_UNORM_PACK16: Self = Self(2);
    pub const B4G4R4A4_UNORM_PACK16: Self = Self(3);
    pub const R5G6B5_UNORM_PACK16: Self = Self(4);
    pub const B5G6R5_UNORM_PACK16: Self = Self(5);
    pub const R5G5B5A1_UNORM_PACK16: Self = Self(6);
    pub const B5G5R5A1_UNORM_PACK16: Self = Self(7);
    pub const A1R5G5B5_UNORM_PACK16: Self = Self(8);
    pub const R8_UNORM: Self = Self(9);
    pub const R8_SNORM: Self = Self(10);
    pub const R8_USCALED: Self = Self(11);
    pub const R8_SSCALED: Self = Self(12);
    pub const R8_UINT: Self = Self(13);
    pub const R8_SINT: Self = Self(14);
    pub const R8_SRGB: Self = Self(15);
    pub const R8G8_UNORM: Self = Self(16);
    pub const R8G8_SNORM: Self = Self(17);
    pub const R8G8_USCALED: Self = Self(18);
    pub const R8G8_SSCALED: Self = Self(19);
    pub const R8G8_UINT: Self = Self(20);
    pub const R8G8_SINT: Self = Self(21);
    pub const R8G8_SRGB: Self = Self(22);
    pub const R8G8B8_UNORM: Self = Self(23);
    pub const R8G8B8_SNORM: Self = Self(24);
    pub const R8G8B8_USCALED: Self = Self(25);
    pub const R8G8B8_SSCALED: Self = Self(26);
    pub const R8G8B8_UINT: Self = Self(27);
    pub const R8G8B8_SINT: Self = Self(28);
    pub const R8G8B8_SRGB: Self = Self(29);
    pub const B8G8R8_UNORM: Self = Self(30);
    pub const B8G8R8_SNORM: Self = Self(31);
    pub const B8G8R8_USCALED: Self = Self(32);
    pub const B8G8R8_SSCALED: Self = Self(33);
    pub const B8G8R8_UINT: Self = Self(34);
    pub const B8G8R8_SINT: Self = Self(35);
    pub const B8G8R8_SRGB: Self = Self(36);
    pub const R8G8B8A8_UNORM: Self = Self(37);
    pub const R8G8B8A8_SNORM: Self = Self(38);
    pub const R8G8B8A8_USCALED: Self = Self(39);
    pub const R8G8B8A8_SSCALED: Self = Self(40);
    pub const R8G8B8A8_UINT: Self = Self(41);
    pub const R8G8B8A8_SINT: Self = Self(42);
    pub const R8G8B8A8_SRGB: Self = Self(43);
    pub const B8G8R8A8_UNORM: Self = Self(44);
    pub const B8G8R8A8_SNORM: Self = Self(45);
    pub const B8G8R8A8_USCALED: Self = Self(46);
    pub const B8G8R8A8_SSCALED: Self = Self(47);
    pub const B8G8R8A8_UINT: Self = Self(48);
    pub const B8G8R8A8_SINT: Self = Self(49);
    pub const B8G8R8A8_SRGB: Self = Self(50);
    pub const A8B8G8R8_UNORM_PACK32: Self = Self(51);
    pub const A8B8G8R8_SNORM_PACK32: Self = Self(52);
    pub const A8B8G8R8_USCALED_PACK32: Self = Self(53);
    pub const A8B8G8R8_SSCALED_PACK32: Self = Self(54);
    pub const A8B8G8R8_UINT_PACK32: Self = Self(55);
    pub const A8B8G8R8_SINT_PACK32: Self = Self(56);
    pub const A8B8G8R8_SRGB_PACK32: Self = Self(57);
    pub const A2R10G10B10_UNORM_PACK32: Self = Self(58);
    pub const A2R10G10B10_SNORM_PACK32: Self = Self(59);
    pub const A2R10G10B10_USCALED_PACK32: Self = Self(60);
    pub const A2R10G10B10_SSCALED_PACK32: Self = Self(61);
    pub const A2R10G10B10_UINT_PACK32: Self = Self(62);
    pub const A2R10G10B10_SINT_PACK32: Self = Self(63);
    pub const A2B10G10R10_UNORM_PACK32: Self = Self(64);
    pub const A2B10G10R10_SNORM_PACK32: Self = Self(65);
    pub const A2B10G10R10_USCALED_PACK32: Self = Self(66);
    pub const A2B10G10R10_SSCALED_PACK32: Self = Self(67);
    pub const A2B10G10R10_UINT_PACK32: Self = Self(68);
    pub const A2B10G10R10_SINT_PACK32: Self = Self(69);
    pub const R16_UNORM: Self = Self(70);
    pub const R16_SNORM: Self = Self(71);
    pub const R16_USCALED: Self = Self(72);
    pub const R16_SSCALED: Self = Self(73);
    pub const R16_UINT: Self = Self(74);
    pub const R16_SINT: Self = Self(75);
    pub const R16_SFLOAT: Self = Self(76);
    pub const R16G16_UNORM: Self = Self(77);
    pub const R16G16_SNORM: Self = Self(78);
    pub const R16G16_USCALED: Self = Self(79);
    pub const R16G16_SSCALED: Self = Self(80);
    pub const R16G16_UINT: Self = Self(81);
    pub const R16G16_SINT: Self = Self(82);
    pub const R16G16_SFLOAT: Self = Self(83);
    pub const R16G16B16_UNORM: Self = Self(84);
    pub const R16G16B16_SNORM: Self = Self(85);
    pub const R16G16B16_USCALED: Self = Self(86);
    pub const R16G16B16_SSCALED: Self = Self(87);
    pub const R16G16B16_UINT: Self = Self(88);
    pub const R16G16B16_SINT: Self = Self(89);
    pub const R16G16B16_SFLOAT: Self = Self(90);
    pub const R16G16B16A16_UNORM: Self = Self(91);
    pub const R16G16B16A16_SNORM: Self = Self(92);
    pub const R16G16B16A16_USCALED: Self = Self(93);
    pub const R16G16B16A16_SSCALED: Self = Self(94);
    pub const R16G16B16A16_UINT: Self = Self(95);
    pub const R16G16B16A16_SINT: Self = Self(96);
    pub const R16G16B16A16_SFLOAT: Self = Self(97);
    pub const R32_UINT: Self = Self(98);
    pub const R32_SINT: Self = Self(99);
    pub const R32_SFLOAT: Self = Self(100);
    pub const R32G32_UINT: Self = Self(101);
    pub const R32G32_SINT: Self = Self(102);
    pub const R32G32_SFLOAT: Self = Self(103);
    pub const R32G32B32_UINT: Self = Self(104);
    pub const R32G32B32_SINT: Self = Self(105);
    pub const R32G32B32_SFLOAT: Self = Self(106);
    pub const R32G32B32A32_UINT: Self = Self(107);
    pub const R32G32B32A32_SINT: Self = Self(108);
    pub const R32G32B32A32_SFLOAT: Self = Self(109);
    pub const R64_UINT: Self = Self(110);
    pub const R64_SINT: Self = Self(111);
    pub const R64_SFLOAT: Self = Self(112);
    pub const R64G64_UINT: Self = Self(113);
    pub const R64G64_SINT: Self = Self(114);
    pub const R64G64_SFLOAT: Self = Self(115);
    pub const R64G64B64_UINT: Self = Self(116);
    pub const R64G64B64_SINT: Self = Self(117);
    pub const R64G64B64_SFLOAT: Self = Self(118);
    pub const R64G64B64A64_UINT: Self = Self(119);
    pub const R64G64B64A64_SINT: Self = Self(120);
    pub const R64G64B64A64_SFLOAT: Self = Self(121);
    pub const B10G11R11_UFLOAT_PACK32: Self = Self(122);
    pub const E5B9G9R9_UFLOAT_PACK32: Self = Self(123);
    pub const D16_UNORM: Self = Self(124);
    pub const X8_D24_UNORM_PACK32: Self = Self(125);
    pub const D32_SFLOAT: Self = Self(126);
    pub const S8_UINT: Self = Self(127);
    pub const D16_UNORM_S8_UINT: Self = Self(128);
    pub const D24_UNORM_S8_UINT: Self = Self(129);
    pub const D32_SFLOAT_S8_UINT: Self = Self(130);
    pub const BC1_RGB_UNORM_BLOCK: Self = Self(131);
    pub const BC1_RGB_SRGB_BLOCK: Self = Self(132);
    pub const BC1_RGBA_UNORM_BLOCK: Self = Self(133);
    pub const BC1_RGBA_SRGB_BLOCK: Self = Self(134);
    pub const BC2_UNORM_BLOCK: Self = Self(135);
    pub const BC2_SRGB_BLOCK: Self = Self(136);
    pub const BC3_UNORM_BLOCK: Self = Self(137);
    pub const BC3_SRGB_BLOCK: Self = Self(138);
    pub const BC4_UNORM_BLOCK: Self = Self(139);
    pub const BC4_SNORM_BLOCK: Self = Self(140);
    pub const BC5_UNORM_BLOCK: Self = Self(141);
    pub const BC5_SNORM_BLOCK: Self = Self(142);
    pub const BC6H_UFLOAT_BLOCK: Self = Self(143);
    pub const BC6H_SFLOAT_BLOCK: Self = Self(144);
    pub const BC7_UNORM_BLOCK: Self = Self(145);
    pub const BC7_SRGB_BLOCK: Self = Self(146);
    pub const ETC2_R8G8B8_UNORM_BLOCK: Self = Self(147);
    pub const ETC2_R8G8B8_SRGB_BLOCK: Self = Self(148);
    pub const ETC2_R8G8B8A1_UNORM_BLOCK: Self = Self(149);
    pub const ETC2_R8G8B8A1_SRGB_BLOCK: Self = Self(150);
    pub const ETC2_R8G8B8A8_UNORM_BLOCK: Self = Self(151);
    pub const ETC2_R8G8B8A8_SRGB_BLOCK: Self = Self(152);
    pub const EAC_R11_UNORM_BLOCK: Self = Self(153);
    pub const EAC_R11_SNORM_BLOCK: Self = Self(154);
    pub const EAC_R11G11_UNORM_BLOCK: Self = Self(155);
    pub const EAC_R11G11_SNORM_BLOCK: Self = Self(156);
    pub const ASTC_4X4_UNORM_BLOCK: Self = Self(157);
    pub const ASTC_4X4_SRGB_BLOCK: Self = Self(158);
    pub const ASTC_5X4_UNORM_BLOCK: Self = Self(159);
    pub const ASTC_5X4_SRGB_BLOCK: Self = Self(160);
    pub const ASTC_5X5_UNORM_BLOCK: Self = Self(161);
    pub const ASTC_5X5_SRGB_BLOCK: Self = Self(162);
    pub const ASTC_6X5_UNORM_BLOCK: Self = Self(163);
    pub const ASTC_6X5_SRGB_BLOCK: Self = Self(164);
    pub const ASTC_6X6_UNORM_BLOCK: Self = Self(165);
    pub const ASTC_6X6_SRGB_BLOCK: Self = Self(166);
    pub const ASTC_8X5_UNORM_BLOCK: Self = Self(167);
    pub const ASTC_8X5_SRGB_BLOCK: Self = Self(168);
    pub const ASTC_8X6_UNORM_BLOCK: Self = Self(169);
    pub const ASTC_8X6_SRGB_BLOCK: Self = Self(170);
    pub const ASTC_8X8_UNORM_BLOCK: Self = Self(171);
    pub const ASTC_8X8_SRGB_BLOCK: Self = Self(172);
    pub const ASTC_10X5_UNORM_BLOCK: Self = Self(173);
    pub const ASTC_10X5_SRGB_BLOCK: Self = Self(174);
    pub const ASTC_10X6_UNORM_BLOCK: Self = Self(175);
    pub const ASTC_10X6_SRGB_BLOCK: Self = Self(176);
    pub const ASTC_10X8_UNORM_BLOCK: Self = Self(177);
    pub const ASTC_10X8_SRGB_BLOCK: Self = Self(178);
    pub const ASTC_10X10_UNORM_BLOCK: Self = Self(179);
    pub const ASTC_10X10_SRGB_BLOCK: Self = Self(180);
    pub const ASTC_12X10_UNORM_BLOCK: Self = Self(181);
    pub const ASTC_12X10_SRGB_BLOCK: Self = Self(182);
    pub const ASTC_12X12_UNORM_BLOCK: Self = Self(183);
    pub const ASTC_12X12_SRGB_BLOCK: Self = Self(184);
    pub const G8B8G8R8_422_UNORM: Self = Self(1000156000);
    pub const B8G8R8G8_422_UNORM: Self = Self(1000156001);
    pub const G8_B8_R8_3PLANE_420_UNORM: Self = Self(1000156002);
    pub const G8_B8R8_2PLANE_420_UNORM: Self = Self(1000156003);
    pub const G8_B8_R8_3PLANE_422_UNORM: Self = Self(1000156004);
    pub const G8_B8R8_2PLANE_422_UNORM: Self = Self(1000156005);
    pub const G8_B8_R8_3PLANE_444_UNORM: Self = Self(1000156006);
    pub const R10X6_UNORM_PACK16: Self = Self(1000156007);
    pub const R10X6G10X6_UNORM_2PACK16: Self = Self(1000156008);
    pub const R10X6G10X6B10X6A10X6_UNORM_4PACK16: Self = Self(1000156009);
    pub const G10X6B10X6G10X6R10X6_422_UNORM_4PACK16: Self = Self(1000156010);
    pub const B10X6G10X6R10X6G10X6_422_UNORM_4PACK16: Self = Self(1000156011);
    pub const G10X6_B10X6_R10X6_3PLANE_420_UNORM_3PACK16: Self =
        Self(1000156012);
    pub const G10X6_B10X6R10X6_2PLANE_420_UNORM_3PACK16: Self =
        Self(1000156013);
    pub const G10X6_B10X6_R10X6_3PLANE_422_UNORM_3PACK16: Self =
        Self(1000156014);
    pub const G10X6_B10X6R10X6_2PLANE_422_UNORM_3PACK16: Self =
        Self(1000156015);
    pub const G10X6_B10X6_R10X6_3PLANE_444_UNORM_3PACK16: Self =
        Self(1000156016);
    pub const R12X4_UNORM_PACK16: Self = Self(1000156017);
    pub const R12X4G12X4_UNORM_2PACK16: Self = Self(1000156018);
    pub const R12X4G12X4B12X4A12X4_UNORM_4PACK16: Self = Self(1000156019);
    pub const G12X4B12X4G12X4R12X4_422_UNORM_4PACK16: Self = Self(1000156020);
    pub const B12X4G12X4R12X4G12X4_422_UNORM_4PACK16: Self = Self(1000156021);
    pub const G12X4_B12X4_R12X4_3PLANE_420_UNORM_3PACK16: Self =
        Self(1000156022);
    pub const G12X4_B12X4R12X4_2PLANE_420_UNORM_3PACK16: Self =
        Self(1000156023);
    pub const G12X4_B12X4_R12X4_3PLANE_422_UNORM_3PACK16: Self =
        Self(1000156024);
    pub const G12X4_B12X4R12X4_2PLANE_422_UNORM_3PACK16: Self =
        Self(1000156025);
    pub const G12X4_B12X4_R12X4_3PLANE_444_UNORM_3PACK16: Self =
        Self(1000156026);
    pub const G16B16G16R16_422_UNORM: Self = Self(1000156027);
    pub const B16G16R16G16_422_UNORM: Self = Self(1000156028);
    pub const G16_B16_R16_3PLANE_420_UNORM: Self = Self(1000156029);
    pub const G16_B16R16_2PLANE_420_UNORM: Self = Self(1000156030);
    pub const G16_B16_R16_3PLANE_422_UNORM: Self = Self(1000156031);
    pub const G16_B16R16_2PLANE_422_UNORM: Self = Self(1000156032);
    pub const G16_B16_R16_3PLANE_444_UNORM: Self = Self(1000156033);
    pub const G8_B8R8_2PLANE_444_UNORM: Self = Self(1000330000);
    pub const G10X6_B10X6R10X6_2PLANE_444_UNORM_3PACK16: Self =
        Self(1000330001);
    pub const G12X4_B12X4R12X4_2PLANE_444_UNORM_3PACK16: Self =
        Self(1000330002);
    pub const G16_B16R16_2PLANE_444_UNORM: Self = Self(1000330003);
    pub const A4R4G4B4_UNORM_PACK16: Self = Self(1000340000);
    pub const A4B4G4R4_UNORM_PACK16: Self = Self(1000340001);
    pub const ASTC_4X4_SFLOAT_BLOCK: Self = Self(1000066000);
    pub const ASTC_5X4_SFLOAT_BLOCK: Self = Self(1000066001);
    pub const ASTC_5X5_SFLOAT_BLOCK: Self = Self(1000066002);
    pub const ASTC_6X5_SFLOAT_BLOCK: Self = Self(1000066003);
    pub const ASTC_6X6_SFLOAT_BLOCK: Self = Self(1000066004);
    pub const ASTC_8X5_SFLOAT_BLOCK: Self = Self(1000066005);
    pub const ASTC_8X6_SFLOAT_BLOCK: Self = Self(1000066006);
    pub const ASTC_8X8_SFLOAT_BLOCK: Self = Self(1000066007);
    pub const ASTC_10X5_SFLOAT_BLOCK: Self = Self(1000066008);
    pub const ASTC_10X6_SFLOAT_BLOCK: Self = Self(1000066009);
    pub const ASTC_10X8_SFLOAT_BLOCK: Self = Self(1000066010);
    pub const ASTC_10X10_SFLOAT_BLOCK: Self = Self(1000066011);
    pub const ASTC_12X10_SFLOAT_BLOCK: Self = Self(1000066012);
    pub const ASTC_12X12_SFLOAT_BLOCK: Self = Self(1000066013);
}

impl Format {
    /// Number of bytes per block
    pub fn bytes(self) -> u64 {
        match self {
            Self::R8_SINT => 1,
            Self::R8_SNORM => 1,
            Self::R8_SRGB => 1,
            Self::R8_UINT => 1,
            Self::R8_UNORM => 1,

            Self::R8G8_SINT => 2,
            Self::R8G8_SNORM => 2,
            Self::R8G8_UINT => 2,
            Self::R8G8_UNORM => 2,
            Self::R16_SFLOAT => 2,
            Self::R16_SINT => 2,
            Self::R16_SNORM => 2,
            Self::R16_UINT => 2,
            Self::R16_UNORM => 2,
            Self::D16_UNORM => 2,
            Self::A1R5G5B5_UNORM_PACK16 => 2,
            Self::B4G4R4A4_UNORM_PACK16 => 2,
            Self::B5G5R5A1_UNORM_PACK16 => 2,
            Self::B5G6R5_UNORM_PACK16 => 2,
            Self::R4G4B4A4_UNORM_PACK16 => 2,
            Self::R5G5B5A1_UNORM_PACK16 => 2,
            Self::R5G6B5_UNORM_PACK16 => 2,

            Self::B8G8R8A8_SINT => 4,
            Self::B8G8R8A8_SNORM => 4,
            Self::B8G8R8A8_SRGB => 4,
            Self::B8G8R8A8_UINT => 4,
            Self::B8G8R8A8_UNORM => 4,
            Self::R8G8B8A8_SINT => 4,
            Self::R8G8B8A8_SNORM => 4,
            Self::R8G8B8A8_SRGB => 4,
            Self::R8G8B8A8_UINT => 4,
            Self::R8G8B8A8_UNORM => 4,
            Self::R16G16_SFLOAT => 4,
            Self::R16G16_SINT => 4,
            Self::R16G16_SNORM => 4,
            Self::R16G16_UINT => 4,
            Self::R16G16_UNORM => 4,
            Self::R32_SFLOAT => 4,
            Self::R32_SINT => 4,
            Self::R32_UINT => 4,
            Self::A2B10G10R10_UINT_PACK32 => 4,
            Self::A2B10G10R10_UNORM_PACK32 => 4,
            Self::A2R10G10B10_UINT_PACK32 => 4,
            Self::A2R10G10B10_UNORM_PACK32 => 4,
            Self::A8B8G8R8_SINT_PACK32 => 4,
            Self::A8B8G8R8_SNORM_PACK32 => 4,
            Self::A8B8G8R8_SRGB_PACK32 => 4,
            Self::A8B8G8R8_UINT_PACK32 => 4,
            Self::A8B8G8R8_UNORM_PACK32 => 4,
            Self::B10G11R11_UFLOAT_PACK32 => 4,
            Self::E5B9G9R9_UFLOAT_PACK32 => 4,
            Self::D24_UNORM_S8_UINT => 4,
            Self::D32_SFLOAT => 4,
            Self::X8_D24_UNORM_PACK32 => 4,

            Self::R16G16B16A16_SFLOAT => 8,
            Self::R16G16B16A16_SINT => 8,
            Self::R16G16B16A16_SNORM => 8,
            Self::R16G16B16A16_UINT => 8,
            Self::R16G16B16A16_UNORM => 8,
            Self::R32G32_SFLOAT => 8,
            Self::R32G32_SINT => 8,
            Self::R32G32_UINT => 8,

            Self::R32G32B32_SFLOAT => 12,
            Self::R32G32B32_SINT => 12,
            Self::R32G32B32_UINT => 12,

            Self::R32G32B32A32_SFLOAT => 16,
            Self::R32G32B32A32_SINT => 16,
            Self::R32G32B32A32_UINT => 16,

            Self::B8G8R8G8_422_UNORM => 4,
            Self::G8B8G8R8_422_UNORM => 4,

            Self::EAC_R11_SNORM_BLOCK => 8,
            Self::EAC_R11_UNORM_BLOCK => 8,
            Self::EAC_R11G11_SNORM_BLOCK => 16,
            Self::EAC_R11G11_UNORM_BLOCK => 16,
            Self::ETC2_R8G8B8_SRGB_BLOCK => 8,
            Self::ETC2_R8G8B8_UNORM_BLOCK => 8,
            Self::ETC2_R8G8B8A1_SRGB_BLOCK => 8,
            Self::ETC2_R8G8B8A1_UNORM_BLOCK => 8,
            Self::ETC2_R8G8B8A8_SRGB_BLOCK => 16,
            Self::ETC2_R8G8B8A8_UNORM_BLOCK => 16,

            _ => panic!("Unimplemented format"),
        }
    }
    /// Number of texels per block
    pub fn texels(self) -> Extent2D {
        const ONE: Extent2D = Extent2D { width: 1, height: 1 };
        const TWO_ONE: Extent2D = Extent2D { width: 2, height: 1 };
        const FOUR_FOUR: Extent2D = Extent2D { width: 4, height: 4 };
        match self {
            Self::R8_SINT => ONE,
            Self::R8_SNORM => ONE,
            Self::R8_SRGB => ONE,
            Self::R8_UINT => ONE,
            Self::R8_UNORM => ONE,
            Self::R8G8_SINT => ONE,
            Self::R8G8_SNORM => ONE,
            Self::R8G8_UINT => ONE,
            Self::R8G8_UNORM => ONE,
            Self::R16_SFLOAT => ONE,
            Self::R16_SINT => ONE,
            Self::R16_SNORM => ONE,
            Self::R16_UINT => ONE,
            Self::R16_UNORM => ONE,
            Self::D16_UNORM => ONE,
            Self::A1R5G5B5_UNORM_PACK16 => ONE,
            Self::B4G4R4A4_UNORM_PACK16 => ONE,
            Self::B5G5R5A1_UNORM_PACK16 => ONE,
            Self::B5G6R5_UNORM_PACK16 => ONE,
            Self::R4G4B4A4_UNORM_PACK16 => ONE,
            Self::R5G5B5A1_UNORM_PACK16 => ONE,
            Self::R5G6B5_UNORM_PACK16 => ONE,
            Self::B8G8R8A8_SINT => ONE,
            Self::B8G8R8A8_SNORM => ONE,
            Self::B8G8R8A8_SRGB => ONE,
            Self::B8G8R8A8_UINT => ONE,
            Self::B8G8R8A8_UNORM => ONE,
            Self::R8G8B8A8_SINT => ONE,
            Self::R8G8B8A8_SNORM => ONE,
            Self::R8G8B8A8_SRGB => ONE,
            Self::R8G8B8A8_UINT => ONE,
            Self::R8G8B8A8_UNORM => ONE,
            Self::R16G16_SFLOAT => ONE,
            Self::R16G16_SINT => ONE,
            Self::R16G16_SNORM => ONE,
            Self::R16G16_UINT => ONE,
            Self::R16G16_UNORM => ONE,
            Self::R32_SFLOAT => ONE,
            Self::R32_SINT => ONE,
            Self::R32_UINT => ONE,
            Self::A2B10G10R10_UINT_PACK32 => ONE,
            Self::A2B10G10R10_UNORM_PACK32 => ONE,
            Self::A2R10G10B10_UINT_PACK32 => ONE,
            Self::A2R10G10B10_UNORM_PACK32 => ONE,
            Self::A8B8G8R8_SINT_PACK32 => ONE,
            Self::A8B8G8R8_SNORM_PACK32 => ONE,
            Self::A8B8G8R8_SRGB_PACK32 => ONE,
            Self::A8B8G8R8_UINT_PACK32 => ONE,
            Self::A8B8G8R8_UNORM_PACK32 => ONE,
            Self::B10G11R11_UFLOAT_PACK32 => ONE,
            Self::E5B9G9R9_UFLOAT_PACK32 => ONE,
            Self::D24_UNORM_S8_UINT => ONE,
            Self::D32_SFLOAT => ONE,
            Self::X8_D24_UNORM_PACK32 => ONE,
            Self::R16G16B16A16_SFLOAT => ONE,
            Self::R16G16B16A16_SINT => ONE,
            Self::R16G16B16A16_SNORM => ONE,
            Self::R16G16B16A16_UINT => ONE,
            Self::R16G16B16A16_UNORM => ONE,
            Self::R32G32_SFLOAT => ONE,
            Self::R32G32_SINT => ONE,
            Self::R32G32_UINT => ONE,
            Self::R32G32B32_SFLOAT => ONE,
            Self::R32G32B32_SINT => ONE,
            Self::R32G32B32_UINT => ONE,
            Self::R32G32B32A32_SFLOAT => ONE,
            Self::R32G32B32A32_SINT => ONE,
            Self::R32G32B32A32_UINT => ONE,

            Self::B8G8R8G8_422_UNORM => TWO_ONE,
            Self::G8B8G8R8_422_UNORM => TWO_ONE,
            Self::EAC_R11_SNORM_BLOCK => FOUR_FOUR,
            Self::EAC_R11_UNORM_BLOCK => FOUR_FOUR,
            Self::EAC_R11G11_SNORM_BLOCK => FOUR_FOUR,
            Self::EAC_R11G11_UNORM_BLOCK => FOUR_FOUR,
            Self::ETC2_R8G8B8_SRGB_BLOCK => FOUR_FOUR,
            Self::ETC2_R8G8B8_UNORM_BLOCK => FOUR_FOUR,
            Self::ETC2_R8G8B8A1_SRGB_BLOCK => FOUR_FOUR,
            Self::ETC2_R8G8B8A1_UNORM_BLOCK => FOUR_FOUR,
            Self::ETC2_R8G8B8A8_SRGB_BLOCK => FOUR_FOUR,
            Self::ETC2_R8G8B8A8_UNORM_BLOCK => FOUR_FOUR,
            _ => panic!("Unimplemented format"),
        }
    }
}

/// Bytes in image size and format. Returns None on overflow.
pub fn image_byte_size_2d(format: Format, extent: Extent2D) -> Option<u64> {
    let block = format.texels();
    let w = (extent.width.checked_add(block.width)? - 1) / block.width;
    let h = (extent.height.checked_add(block.height)? - 1) / block.height;
    let blocks = (w as u64).checked_mul(h as u64)?;
    blocks.checked_mul(format.bytes())
}
/// Bytes in image size and format. Returns None on overflow.
pub fn image_byte_size_3d(format: Format, extent: Extent3D) -> Option<u64> {
    let size = image_byte_size_2d(
        format,
        Extent2D { width: extent.width, height: extent.height },
    )?;
    size.checked_mul(extent.depth as u64)
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct ImageLayout(u32);
impl ImageLayout {
    pub const UNDEFINED: Self = Self(0);
    pub const GENERAL: Self = Self(1);
    pub const COLOR_ATTACHMENT_OPTIMAL: Self = Self(2);
    pub const DEPTH_STENCIL_ATTACHMENT_OPTIMAL: Self = Self(3);
    pub const DEPTH_STENCIL_READ_ONLY_OPTIMAL: Self = Self(4);
    pub const SHADER_READ_ONLY_OPTIMAL: Self = Self(5);
    pub const TRANSFER_SRC_OPTIMAL: Self = Self(6);
    pub const TRANSFER_DST_OPTIMAL: Self = Self(7);
    pub const PREINITIALIZED: Self = Self(8);
    pub const DEPTH_READ_ONLY_STENCIL_ATTACHMENT_OPTIMAL: Self =
        Self(1000117000);
    pub const DEPTH_ATTACHMENT_STENCIL_READ_ONLY_OPTIMAL: Self =
        Self(1000117001);
    pub const DEPTH_ATTACHMENT_OPTIMAL: Self = Self(1000241000);
    pub const DEPTH_READ_ONLY_OPTIMAL: Self = Self(1000241001);
    pub const STENCIL_ATTACHMENT_OPTIMAL: Self = Self(1000241002);
    pub const STENCIL_READ_ONLY_OPTIMAL: Self = Self(1000241003);
    pub const READ_ONLY_OPTIMAL: Self = Self(1000314000);
    pub const ATTACHMENT_OPTIMAL: Self = Self(1000314001);
    pub const PRESENT_SRC_KHR: Self = Self(1000001002);
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    #[doc = crate::man_link!(VkImageAspectFlagBits)]
    pub struct ImageAspectFlags: u32 {
        const COLOR = 0x01;
        const DEPTH = 0x02;
        const STENCIL = 0x04;
        const METADATA = 0x08;
        const PLANE_0 = 0x10;
        const PLANE_1 = 0x20;
        const PLANE_2 = 0x40;
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    #[doc = crate::man_link!(VkCommandPoolCreateFlagBits)]
    pub struct CommandPoolCreateFlags: u32 {
        const TRANSIENT = 0x1;
        const RESET_COMMAND_BUFFER = 0x2;
        const PROTECTED = 0x4;
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    #[doc = crate::man_link!(VkCommandPoolResetFlagBits)]
    pub struct CommandPoolResetFlags: u32 {
        const RELEASE_RESOURCES = 0x1;
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[doc = crate::man_link!(VkCommandBufferLevel)]
pub struct CommandBufferLevel(u32);
impl CommandBufferLevel {
    pub const PRIMARY: Self = Self(0);
    pub const SECONDARY: Self = Self(1);
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    /// Reserved
    pub struct QueryControlFlags: u32 {}
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    /// Reserved
    pub struct QueryPipelineStatisticFlags: u32 {}
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    #[doc = crate::man_link!(VkCommandBufferUsageFlagBits)]
    pub struct CommandBufferUsageFlags: u32 {
        const ONE_TIME_SUBMIT = 0x1;
        const RENDER_PASS_CONTINUE = 0x2;
        const SIMULTANEOUS_USE = 0x4;
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[doc = crate::man_link!(VkAttachmentLoadOp)]
pub struct AttachmentLoadOp(u32);
impl AttachmentLoadOp {
    pub const LOAD: Self = Self(0);
    pub const CLEAR: Self = Self(1);
    pub const DONT_CARE: Self = Self(2);
    pub const NONE_EXT: Self = Self(1000400000);
}

impl Default for AttachmentLoadOp {
    fn default() -> Self {
        Self::DONT_CARE
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[doc = crate::man_link!(VkAttachmentStoreOp)]
pub struct AttachmentStoreOp(u32);
impl AttachmentStoreOp {
    pub const STORE: Self = Self(0);
    pub const DONT_CARE: Self = Self(1);
    pub const NONE: Self = Self(1000301000);
}

impl Default for AttachmentStoreOp {
    fn default() -> Self {
        Self::DONT_CARE
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    /// Reserved
    pub struct SubpassDescriptionFlags: u32 {}
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[doc = crate::man_link!(VkPipelineBindPoint)]
pub struct PipelineBindPoint(u32);
impl PipelineBindPoint {
    pub const GRAPHICS: Self = Self(0);
    pub const COMPUTE: Self = Self(1);
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    #[doc = crate::man_link!(VkFramebufferCreateFlagBits)]
    pub struct FramebufferCreateFlags: u32 {
        const IMAGELESS = 0x01;
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    /// Reserved
    pub struct ShaderModuleCreateFlags: u32 {}
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    /// Reserved
    pub struct PipelineCacheCreateFlags: u32 {}
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    #[doc = crate::man_link!(VkPipelineShaderStageCreateFlagBits)]
    pub struct PipelineShaderStageCreateFlags: u32 {
        const ALLOW_VARYING_SUBGROUP_SIZE = 0x1;
        const REQUIRE_FULL_SUBGROUPS = 0x2;
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    /// Reserved
    pub struct PipelineVertexInputStateCreateFlags: u32 {}
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[doc = crate::man_link!(VkVertexInputRate)]
pub struct VertexInputRate(u32);
impl VertexInputRate {
    pub const VERTEX: Self = Self(0);
    pub const INSTANCE: Self = Self(1);
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    /// Reserved
    pub struct PipelineInputAssemblyStateCreateFlags: u32 {}
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[doc = crate::man_link!(VkPrimitiveTopology)]
pub struct PrimitiveTopology(u32);
impl PrimitiveTopology {
    pub const POINT_LIST: Self = Self(0);
    pub const LINE_LIST: Self = Self(1);
    pub const LINE_STRIP: Self = Self(2);
    pub const TRIANGLE_LIST: Self = Self(3);
    pub const TRIANGLE_STRIP: Self = Self(4);
    pub const TRIANGLE_FAN: Self = Self(5);
    pub const LINE_LIST_WITH_ADJACENCY: Self = Self(6);
    pub const LINE_STRIP_WITH_ADJACENCY: Self = Self(7);
    pub const TRIANGLE_LIST_WITH_ADJACENCY: Self = Self(8);
    pub const TRIANGLE_STRIP_WITH_ADJACENCY: Self = Self(9);
    pub const PATCH_LIST: Self = Self(10);
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    /// Reserved
    pub struct PipelineTesselationStateCreateFlags: u32 {}
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    /// Reserved
    pub struct PipelineViewportStateCreateFlags: u32 {}
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    /// Reserved
    pub struct PipelineRasterizationStateCreateFlags: u32 {}
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[doc = crate::man_link!(VkPolygonMode)]
pub struct PolygonMode(u32);
impl PolygonMode {
    pub const FILL: Self = Self(0);
    pub const LINE: Self = Self(1);
    pub const POINT: Self = Self(2);
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    #[doc = crate::man_link!(VkCullModeFlagBits)]
    pub struct CullModeFlags: u32 {
        const FRONT = 0x1;
        const BACK = 0x2;
        const FRONT_AND_BACK = 0x3;
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[doc = crate::man_link!(VkFrontFace)]
pub struct FrontFace(u32);
impl FrontFace {
    pub const COUNTER_CLOCKWISE: Self = Self(0);
    pub const CLOCKWISE: Self = Self(1);
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    /// Reserved
    pub struct PipelineMultisampleStateCreateFlags: u32 {}
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    /// Reserved
    pub struct PipelineDepthStencilStateCreateFlags: u32 {}
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[doc = crate::man_link!(VkCompareOp)]
pub struct CompareOp(u32);
impl CompareOp {
    pub const NEVER: Self = Self(0);
    pub const LESS: Self = Self(1);
    pub const EQUAL: Self = Self(2);
    pub const LESS_OR_EQUAL: Self = Self(3);
    pub const GREATER: Self = Self(4);
    pub const NOT_EQUAL: Self = Self(5);
    pub const GREATER_OR_EQUAL: Self = Self(6);
    pub const ALWAYS: Self = Self(7);
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[doc = crate::man_link!(VkStencilOp)]
pub struct StencilOp(u32);
impl StencilOp {
    pub const KEEP: Self = Self(0);
    pub const ZERO: Self = Self(1);
    pub const REPLACE: Self = Self(2);
    pub const INCREMENT_AND_CLAMP: Self = Self(3);
    pub const DECREMENT_AND_CLAMP: Self = Self(4);
    pub const INVERT: Self = Self(5);
    pub const INCREMENT_AND_WRAP: Self = Self(6);
    pub const DECREMENT_AND_WRAP: Self = Self(7);
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[doc = crate::man_link!(VkBlendFactor)]
pub struct BlendFactor(u32);
impl BlendFactor {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1);
    pub const SRC_COLOR: Self = Self(2);
    pub const ONE_MINUS_SRC_COLOR: Self = Self(3);
    pub const DST_COLOR: Self = Self(4);
    pub const ONE_MINUS_DST_COLOR: Self = Self(5);
    pub const SRC_ALPHA: Self = Self(6);
    pub const ONE_MINUS_SRC_ALPHA: Self = Self(7);
    pub const DST_ALPHA: Self = Self(8);
    pub const ONE_MINUS_DST_ALPHA: Self = Self(9);
    pub const CONSTANT_COLOR: Self = Self(10);
    pub const ONE_MINUS_CONSTANT_COLOR: Self = Self(11);
    pub const CONSTANT_ALPHA: Self = Self(12);
    pub const ONE_MINUS_CONSTANT_ALPHA: Self = Self(13);
    pub const SRC_ALPHA_SATURATE: Self = Self(14);
    pub const SRC1_COLOR: Self = Self(15);
    pub const ONE_MINUS_SRC1_COLOR: Self = Self(16);
    pub const SRC1_ALPHA: Self = Self(17);
    pub const ONE_MINUS_SRC1_ALPHA: Self = Self(18);
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[doc = crate::man_link!(VkBlendOp)]
pub struct BlendOp(u32);
impl BlendOp {
    pub const ADD: Self = Self(0);
    pub const SUBTRACT: Self = Self(1);
    pub const REVERSE_SUBTRACT: Self = Self(2);
    pub const MIN: Self = Self(3);
    pub const MAX: Self = Self(4);
    pub const ZERO_EXT: Self = Self(1000148000);
    pub const SRC_EXT: Self = Self(1000148001);
    pub const DST_EXT: Self = Self(1000148002);
    pub const SRC_OVER_EXT: Self = Self(1000148003);
    pub const DST_OVER_EXT: Self = Self(1000148004);
    pub const SRC_IN_EXT: Self = Self(1000148005);
    pub const DST_IN_EXT: Self = Self(1000148006);
    pub const SRC_OUT_EXT: Self = Self(1000148007);
    pub const DST_OUT_EXT: Self = Self(1000148008);
    pub const SRC_ATOP_EXT: Self = Self(1000148009);
    pub const DST_ATOP_EXT: Self = Self(1000148010);
    pub const XOR_EXT: Self = Self(1000148011);
    pub const MULTIPLY_EXT: Self = Self(1000148012);
    pub const SCREEN_EXT: Self = Self(1000148013);
    pub const OVERLAY_EXT: Self = Self(1000148014);
    pub const DARKEN_EXT: Self = Self(1000148015);
    pub const LIGHTEN_EXT: Self = Self(1000148016);
    pub const COLORDODGE_EXT: Self = Self(1000148017);
    pub const COLORBURN_EXT: Self = Self(1000148018);
    pub const HARDLIGHT_EXT: Self = Self(1000148019);
    pub const SOFTLIGHT_EXT: Self = Self(1000148020);
    pub const DIFFERENCE_EXT: Self = Self(1000148021);
    pub const EXCLUSION_EXT: Self = Self(1000148022);
    pub const INVERT_EXT: Self = Self(1000148023);
    pub const INVERT_RGB_EXT: Self = Self(1000148024);
    pub const LINEARDODGE_EXT: Self = Self(1000148025);
    pub const LINEARBURN_EXT: Self = Self(1000148026);
    pub const VIVIDLIGHT_EXT: Self = Self(1000148027);
    pub const LINEARLIGHT_EXT: Self = Self(1000148028);
    pub const PINLIGHT_EXT: Self = Self(1000148029);
    pub const HARDMIX_EXT: Self = Self(1000148030);
    pub const HSL_HUE_EXT: Self = Self(1000148031);
    pub const HSL_SATURATION_EXT: Self = Self(1000148032);
    pub const HSL_COLOR_EXT: Self = Self(1000148033);
    pub const HSL_LUMINOSITY_EXT: Self = Self(1000148034);
    pub const PLUS_EXT: Self = Self(1000148035);
    pub const PLUS_CLAMPED_EXT: Self = Self(1000148036);
    pub const PLUS_CLAMPED_ALPHA_EXT: Self = Self(1000148037);
    pub const PLUS_DARKER_EXT: Self = Self(1000148038);
    pub const MINUS_EXT: Self = Self(1000148039);
    pub const MINUS_CLAMPED_EXT: Self = Self(1000148040);
    pub const CONTRAST_EXT: Self = Self(1000148041);
    pub const INVERT_OVG_EXT: Self = Self(1000148042);
    pub const RED_EXT: Self = Self(1000148043);
    pub const GREEN_EXT: Self = Self(1000148044);
    pub const BLUE_EXT: Self = Self(1000148045);
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    #[doc = crate::man_link!(VkColorComponentFlagBits)]
    pub struct ColorComponentFlags: u32 {
        const R = 0x1;
        const G = 0x2;
        const B = 0x4;
        const A = 0x8;
        const RGBA = 0xF;
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    /// Reserved
    pub struct PipelineColorBlendStateCreateFlags: u32 {}
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    #[doc = crate::man_link!(VkPipelineCreateFlagBits)]
    pub struct PipelineCreateFlags: u32 {
        const DISABLE_OPTIMIZATION = 0x00000001;
        const ALLOW_DERIVATIVES = 0x00000002;
        const DERIVATIVE = 0x00000004;
        const VIEW_INDEX_FROM_DEVICE_INDEX = 0x00000008;
        const DISPATCH_BASE = 0x00000010;
        const FAIL_ON_PIPELINE_COMPILE_REQUIRED = 0x00000100;
        const EARLY_RETURN_ON_FAILURE = 0x00000200;
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[doc = crate::man_link!(VkLogicOp)]
pub struct LogicOp(u32);
impl LogicOp {
    pub const CLEAR: Self = Self(0);
    pub const AND: Self = Self(1);
    pub const AND_REVERSE: Self = Self(2);
    pub const COPY: Self = Self(3);
    pub const AND_INVERTED: Self = Self(4);
    pub const NO_OP: Self = Self(5);
    pub const XOR: Self = Self(6);
    pub const OR: Self = Self(7);
    pub const NOR: Self = Self(8);
    pub const EQUIVALENT: Self = Self(9);
    pub const INVERT: Self = Self(10);
    pub const OR_REVERSE: Self = Self(11);
    pub const COPY_INVERTED: Self = Self(12);
    pub const OR_INVERTED: Self = Self(13);
    pub const NAND: Self = Self(14);
    pub const SET: Self = Self(15);
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    /// Reserved
    pub struct DynamicStateCreateFlags: u32 {}
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[doc = crate::man_link!(VkDynamicState)]
pub struct DynamicState(u32);
impl DynamicState {
    pub const VIEWPORT: Self = Self(0);
    pub const SCISSOR: Self = Self(1);
    pub const LINE_WIDTH: Self = Self(2);
    pub const DEPTH_BIAS: Self = Self(3);
    pub const BLEND_CONSTANTS: Self = Self(4);
    pub const DEPTH_BOUNDS: Self = Self(5);
    pub const STENCIL_COMPARE_MASK: Self = Self(6);
    pub const STENCIL_WRITE_MASK: Self = Self(7);
    pub const STENCIL_REFERENCE: Self = Self(8);
    pub const CULL_MODE: Self = Self(1000267000);
    pub const FRONT_FACE: Self = Self(1000267001);
    pub const PRIMITIVE_TOPOLOGY: Self = Self(1000267002);
    pub const VIEWPORT_WITH_COUNT: Self = Self(1000267003);
    pub const SCISSOR_WITH_COUNT: Self = Self(1000267004);
    pub const VERTEX_INPUT_BINDING_STRIDE: Self = Self(1000267005);
    pub const DEPTH_TEST_ENABLE: Self = Self(1000267006);
    pub const DEPTH_WRITE_ENABLE: Self = Self(1000267007);
    pub const DEPTH_COMPARE_OP: Self = Self(1000267008);
    pub const DEPTH_BOUNDS_TEST_ENABLE: Self = Self(1000267009);
    pub const STENCIL_TEST_ENABLE: Self = Self(1000267010);
    pub const STENCIL_OP: Self = Self(1000267011);
    pub const RASTERIZER_DISCARD_ENABLE: Self = Self(1000377001);
    pub const DEPTH_BIAS_ENABLE: Self = Self(1000377002);
    pub const PRIMITIVE_RESTART_ENABLE: Self = Self(1000377004);
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    /// Reserved
    pub struct RenderPassCreateFlags: u32 {}
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[doc = crate::man_link!(VkColorSpaceKHR)]
pub struct ColorSpaceKHR(u32);
impl ColorSpaceKHR {
    pub const SRGB_NONLINEAR_KHR: Self = Self(0);
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    #[doc = crate::man_link!(VkDescriptorSetLayoutCreateFlagBits)]
    pub struct DescriptorSetLayoutCreateFlags: u32 {
        const UPDATE_AFTER_BIND_POOL = 0x2;
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    #[doc = crate::man_link!(VkDescriptorPoolCreateFlagBits)]
    pub struct DescriptorPoolCreateFlags: u32 {
        const CREATE_FREE_DESCRIPTOR_SET = 0x1;
        const CREATE_UPDATE_AFTER_BIND = 0x2;
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    /// Reserved
    pub struct DescriptorPoolResetFlags: u32 {}
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    #[doc = crate::man_link!(VkPipelineLayoutCreateFlagBits)]
    pub struct PipelineLayoutCreateFlags: u32 {
        const INDEPENDENT_SETS_EXT = 0x2;
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    /// Reserved
    pub struct SamplerCreateFlags: u32 {}
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[doc = crate::man_link!(VkFilter)]
pub struct Filter(u32);
impl Filter {
    pub const NEAREST: Self = Self(0);
    pub const LINEAR: Self = Self(1);
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[doc = crate::man_link!(VkSamplerMipmapMode)]
pub struct SamplerMipmapMode(u32);
impl SamplerMipmapMode {
    pub const NEAREST: Self = Self(0);
    pub const LINEAR: Self = Self(1);
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[doc = crate::man_link!(VkSamplerAddressMode)]
pub struct SamplerAddressMode(u32);
impl SamplerAddressMode {
    pub const REPEAT: Self = Self(0);
    pub const MIRRORED_REPEAT: Self = Self(1);
    pub const CLAMP_TO_EDGE: Self = Self(2);
    pub const CLAMP_TO_BORDER: Self = Self(3);
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[doc = crate::man_link!(VkBorderColor)]
pub struct BorderColor(u32);
impl BorderColor {
    pub const FLOAT_TRANSPARENT_BLACK: Self = Self(0);
    pub const INT_TRANSPARENT_BLACK: Self = Self(1);
    pub const FLOAT_OPAQUE_BLACK: Self = Self(2);
    pub const INT_OPAQUE_BLACK: Self = Self(3);
    pub const FLOAT_OPAQUE_WHITE: Self = Self(4);
    pub const INT_OPAQUE_WHITE: Self = Self(5);
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[doc = crate::man_link!(VkDescriptorType)]
pub struct DescriptorType(u32);
impl DescriptorType {
    pub const SAMPLER: Self = Self(0);
    /// Can only be used with immutable samplers at the moment
    pub const COMBINED_IMAGE_SAMPLER: Self = Self(1);
    pub const SAMPLED_IMAGE: Self = Self(2);
    pub const STORAGE_IMAGE: Self = Self(3);
    pub const UNIFORM_TEXEL_BUFFER: Self = Self(4);
    pub const STORAGE_TEXEL_BUFFER: Self = Self(5);
    pub const UNIFORM_BUFFER: Self = Self(6);
    pub const STORAGE_BUFFER: Self = Self(7);
    pub const UNIFORM_BUFFER_DYNAMIC: Self = Self(8);
    pub const STORAGE_BUFFER_DYNAMIC: Self = Self(9);
    pub const INPUT_ATTACHMENT: Self = Self(10);
    // TODO: Extension object
    // pub const INLINE_UNIFORM_BLOCK: Self = Self(1000138000);
}

impl Default for DescriptorType {
    fn default() -> Self {
        Self::UNIFORM_BUFFER
    }
}

impl DescriptorType {
    pub fn supports_buffer_usage(self, usage: BufferUsageFlags) -> bool {
        use BufferUsageFlags as Buf;
        let bit = match self {
            Self::UNIFORM_TEXEL_BUFFER => Buf::UNIFORM_TEXEL_BUFFER,
            Self::STORAGE_TEXEL_BUFFER => Buf::STORAGE_TEXEL_BUFFER,
            Self::UNIFORM_BUFFER => Buf::UNIFORM_BUFFER,
            Self::STORAGE_BUFFER => Buf::STORAGE_BUFFER,
            Self::UNIFORM_BUFFER_DYNAMIC => Buf::UNIFORM_BUFFER,
            Self::STORAGE_BUFFER_DYNAMIC => Buf::STORAGE_BUFFER,
            _ => Buf::empty(),
        };
        usage.intersects(bit)
    }
    pub fn supports_image_usage(self, usage: ImageUsageFlags) -> bool {
        use ImageUsageFlags as Img;
        let bit = match self {
            Self::COMBINED_IMAGE_SAMPLER => Img::SAMPLED,
            Self::SAMPLED_IMAGE => Img::SAMPLED,
            Self::STORAGE_IMAGE => Img::STORAGE,
            Self::INPUT_ATTACHMENT => Img::INPUT_ATTACHMENT,
            _ => Img::empty(),
        };
        usage.intersects(bit)
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    #[doc = crate::man_link!(VkShaderStageFlagBits)]
    pub struct ShaderStageFlags: u32 {
        const VERTEX = 0x01;
        const TESSELLATION_CONTROL = 0x02;
        const TESSELLATION_EVALUATION = 0x04;
        const GEOMETRY = 0x08;
        const FRAGMENT = 0x10;
        const COMPUTE = 0x20;
        const ALL_GRAPHICS = 0x1F;
        const ALL = 0x7FFFFFFF;
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    #[doc = crate::man_link!(VkMemoryPropertyFlagBits)]
    pub struct MemoryPropertyFlags: u32 {
        const DEVICE_LOCAL = 0x01;
        const HOST_VISIBLE = 0x02;
        const HOST_COHERENT = 0x04;
        const HOST_CACHED = 0x08;
        const LAZILY_ALLOCATED = 0x10;
        const PROTECTED = 0x20;
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    #[doc = crate::man_link!(VkMemoryHeapFlagBits)]
    pub struct MemoryHeapFlags: u32 {
        const DEVICE_LOCAL = 0x1;
        const MULTI_INSTANCE = 0x2;
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    /// Reserved
    pub struct MemoryMapFlags: u32 {}
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    #[doc = crate::man_link!(VkSwapchainCreateFlagBits)]
    pub struct SwapchainCreateFlagsKHR: u32 {
        const SPLIT_INSTANCE_BIND_REGIONS = 0x1;
        const PROTECTED = 0x2;
        const MUTABLE_FORMAT = 0x4;
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[doc = crate::man_link!(VkSharingMode)]
pub struct SharingMode(u32);
impl SharingMode {
    pub const EXCLUSIVE: Self = Self(0);
    pub const CONCURRENT: Self = Self(1);
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[doc = crate::man_link!(VkPresentModeKHR)]
pub struct PresentModeKHR(u32);
impl PresentModeKHR {
    pub const IMMEDIATE: Self = Self(0);
    pub const MAILBOX: Self = Self(1);
    pub const FIFO: Self = Self(2);
    pub const FIFO_RELAXED: Self = Self(3);
    pub const SHARED_DEMAND_REFRESH: Self = Self(1000111000);
    pub const SHARED_CONTINUOUS_REFRESH: Self = Self(1000111001);
}

impl Default for PresentModeKHR {
    fn default() -> Self {
        Self::FIFO
    }
}
