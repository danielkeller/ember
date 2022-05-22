macro_rules! flags {
    ($name: ident, [$($member:ident),*]) => {
        impl Default for $name {
            fn default() -> Self {
                Self(0)
            }
        }
        impl $name {
            #[inline]
            pub const fn empty() -> Self {
                Self(0)
            }
            #[inline]
            pub const fn is_empty(self) -> bool {
                self.0 == Self::empty().0
            }
        }
        impl ::std::ops::BitOr for $name {
            type Output = Self;
            #[inline]
            fn bitor(self, rhs: Self) -> Self {
                Self(self.0 | rhs.0)
            }
        }
        impl ::std::ops::BitOrAssign for $name {
            #[inline]
            fn bitor_assign(&mut self, rhs: Self) {
                *self = *self | rhs
            }
        }
        impl ::std::ops::BitAnd for $name {
            type Output = Self;
            #[inline]
            fn bitand(self, rhs: Self) -> Self {
                Self(self.0 & rhs.0)
            }
        }
        impl ::std::ops::BitAndAssign for $name {
            #[inline]
            fn bitand_assign(&mut self, rhs: Self) {
                *self = *self & rhs
            }
        }
        impl ::std::ops::BitXor for $name {
            type Output = Self;
            #[inline]
            fn bitxor(self, rhs: Self) -> Self {
                Self(self.0 ^ rhs.0)
            }
        }
        impl ::std::ops::BitXorAssign for $name {
            #[inline]
            fn bitxor_assign(&mut self, rhs: Self) {
                *self = *self ^ rhs
            }
        }
        impl ::std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                #[allow(unused_mut)]
                let mut remaining = *self;
                $(if !(*self & $name::$member).is_empty() {
                    if remaining != *self { f.write_str(" | ")?; }
                    f.write_str("vk::")?;
                    f.write_str(stringify!($name))?;
                    f.write_str("::")?;
                    f.write_str(stringify!($member))?;
                    remaining ^= $name::$member;
                })*
                if !remaining.is_empty() {
                    if remaining != *self { f.write_str(" | ")?; }
                    f.write_str("vk::")?;
                    f.write_str(stringify!($name))?;
                    f.write_str("(")?;
                    remaining.0.fmt(f)?;
                    f.write_str(")")?;
                }
                if self.is_empty() {
                    f.write_str("vk::")?;
                    f.write_str(stringify!($name))?;
                    f.write_str("::empty()")?;
                }
                Ok(())
            }
        }
    };
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct InstanceCreateFlags(u32);
flags!(InstanceCreateFlags, []);

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq)]
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
pub struct SampleCount(u32);
impl SampleCount {
    pub const _1: SampleCount = SampleCount(0x01);
    pub const _2: SampleCount = SampleCount(0x02);
    pub const _4: SampleCount = SampleCount(0x04);
    pub const _8: SampleCount = SampleCount(0x08);
    pub const _16: SampleCount = SampleCount(0x10);
    pub const _32: SampleCount = SampleCount(0x20);
    pub const _64: SampleCount = SampleCount(0x40);
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SampleCountFlags(u32);
impl SampleCountFlags {
    pub const _1: SampleCountFlags = SampleCountFlags(0x01);
    pub const _2: SampleCountFlags = SampleCountFlags(0x02);
    pub const _4: SampleCountFlags = SampleCountFlags(0x04);
    pub const _8: SampleCountFlags = SampleCountFlags(0x08);
    pub const _16: SampleCountFlags = SampleCountFlags(0x10);
    pub const _32: SampleCountFlags = SampleCountFlags(0x20);
    pub const _64: SampleCountFlags = SampleCountFlags(0x40);
}
flags!(SampleCountFlags, [_1, _2, _4, _8, _16, _32, _64]);

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
        Self(bit.0)
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct QueueFlags(u32);
impl QueueFlags {
    pub const GRAPHICS: QueueFlags = QueueFlags(0x01);
    pub const COMPUTE: QueueFlags = QueueFlags(0x02);
    pub const TRANSFER: QueueFlags = QueueFlags(0x04);
    pub const SPARSE_BINDING: QueueFlags = QueueFlags(0x08);
    pub const PROTECTED: QueueFlags = QueueFlags(0x10);
}
flags!(QueueFlags, [GRAPHICS, COMPUTE, TRANSFER, SPARSE_BINDING, PROTECTED]);

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeviceCreateFlags(u32);
flags!(DeviceCreateFlags, []);

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeviceQueueCreateFlags(u32);
impl DeviceQueueCreateFlags {
    pub const PROTECTED: DeviceQueueCreateFlags = DeviceQueueCreateFlags(0x1);
}
flags!(DeviceQueueCreateFlags, [PROTECTED]);

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PipelineStageFlags(u32);
impl PipelineStageFlags {
    pub const NONE: Self = Self(0);
    pub const TOP_OF_PIPE: Self = Self(0x00000001);
    pub const DRAW_INDIRECT: Self = Self(0x00000002);
    pub const VERTEX_INPUT: Self = Self(0x00000004);
    pub const VERTEX_SHADER: Self = Self(0x00000008);
    pub const TESSELLATION_CONTROL_SHADER: Self = Self(0x00000010);
    pub const TESSELLATION_EVALUATION_SHADER: Self = Self(0x00000020);
    pub const GEOMETRY_SHADER: Self = Self(0x00000040);
    pub const FRAGMENT_SHADER: Self = Self(0x00000080);
    pub const EARLY_FRAGMENT_TESTS: Self = Self(0x00000100);
    pub const LATE_FRAGMENT_TESTS: Self = Self(0x00000200);
    pub const COLOR_ATTACHMENT_OUTPUT: Self = Self(0x00000400);
    pub const COMPUTE_SHADER: Self = Self(0x00000800);
    pub const TRANSFER: Self = Self(0x00001000);
    pub const BOTTOM_OF_PIPE: Self = Self(0x00002000);
    pub const HOST: Self = Self(0x00004000);
    pub const ALL_GRAPHICS: Self = Self(0x00008000);
    pub const ALL_COMMANDS: Self = Self(0x00010000);
    pub const TRANSFORM_FEEDBACK_EXT: Self = Self(0x01000000);
    pub const CONDITIONAL_RENDERING_EXT: Self = Self(0x00040000);
    pub const ACCELERATION_STRUCTURE_BUILD_KHR: Self = Self(0x02000000);
    pub const RAY_TRACING_SHADER_KHR: Self = Self(0x00200000);
    pub const FRAGMENT_DENSITY_PROCESS_EXT: Self = Self(0x00800000);
    pub const FRAGMENT_SHADING_RATE_ATTACHMENT_KHR: Self = Self(0x00400000);
}
flags!(
    PipelineStageFlags,
    [
        TOP_OF_PIPE,
        DRAW_INDIRECT,
        VERTEX_INPUT,
        VERTEX_SHADER,
        TESSELLATION_CONTROL_SHADER,
        TESSELLATION_EVALUATION_SHADER,
        GEOMETRY_SHADER,
        FRAGMENT_SHADER,
        EARLY_FRAGMENT_TESTS,
        LATE_FRAGMENT_TESTS,
        COLOR_ATTACHMENT_OUTPUT,
        COMPUTE_SHADER,
        TRANSFER,
        BOTTOM_OF_PIPE,
        HOST,
        ALL_GRAPHICS,
        ALL_COMMANDS,
        TRANSFORM_FEEDBACK_EXT,
        CONDITIONAL_RENDERING_EXT,
        ACCELERATION_STRUCTURE_BUILD_KHR,
        RAY_TRACING_SHADER_KHR,
        FRAGMENT_DENSITY_PROCESS_EXT,
        FRAGMENT_SHADING_RATE_ATTACHMENT_KHR
    ]
);

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct DependencyFlags(u32);
impl DependencyFlags {
    pub const BY_REGION: Self = Self(0x1);
    pub const DEVICE_GROUP: Self = Self(0x4);
    pub const VIEW_LOCAL: Self = Self(0x2);
}
flags!(DependencyFlags, [BY_REGION, DEVICE_GROUP, VIEW_LOCAL]);

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct AccessFlags(u32);
impl AccessFlags {
    pub const INDIRECT_COMMAND_READ: Self = Self(0x00001);
    pub const INDEX_READ: Self = Self(0x00002);
    pub const VERTEX_ATTRIBUTE_READ: Self = Self(0x00004);
    pub const UNIFORM_READ: Self = Self(0x00008);
    pub const INPUT_ATTACHMENT_READ: Self = Self(0x00010);
    pub const SHADER_READ: Self = Self(0x00020);
    pub const SHADER_WRITE: Self = Self(0x00040);
    pub const COLOR_ATTACHMENT_READ: Self = Self(0x00080);
    pub const COLOR_ATTACHMENT_WRITE: Self = Self(0x00100);
    pub const DEPTH_STENCIL_ATTACHMENT_READ: Self = Self(0x00200);
    pub const DEPTH_STENCIL_ATTACHMENT_WRITE: Self = Self(0x00400);
    pub const TRANSFER_READ: Self = Self(0x00800);
    pub const TRANSFER_WRITE: Self = Self(0x01000);
    pub const HOST_READ: Self = Self(0x02000);
    pub const HOST_WRITE: Self = Self(0x04000);
    pub const MEMORY_READ: Self = Self(0x08000);
    pub const MEMORY_WRITE: Self = Self(0x10000);
}
flags!(
    AccessFlags,
    [
        INDIRECT_COMMAND_READ,
        INDEX_READ,
        VERTEX_ATTRIBUTE_READ,
        UNIFORM_READ,
        INPUT_ATTACHMENT_READ,
        SHADER_READ,
        SHADER_WRITE,
        COLOR_ATTACHMENT_READ,
        COLOR_ATTACHMENT_WRITE,
        DEPTH_STENCIL_ATTACHMENT_READ,
        DEPTH_STENCIL_ATTACHMENT_WRITE,
        TRANSFER_READ,
        TRANSFER_WRITE,
        HOST_READ,
        HOST_WRITE,
        MEMORY_READ,
        MEMORY_WRITE
    ]
);

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct AttachmentDescriptionFlags(u32);
impl AttachmentDescriptionFlags {
    pub const MAY_ALIAS: Self = Self(0x1);
}
flags!(AttachmentDescriptionFlags, [MAY_ALIAS]);

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct FenceCreateFlags(u32);
impl FenceCreateFlags {
    pub const SIGNALLED: FenceCreateFlags = FenceCreateFlags(0x1);
}
flags!(FenceCreateFlags, [SIGNALLED]);

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SemaphoreCreateFlags(u32);
flags!(SemaphoreCreateFlags, []);

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageViewCreateFlags(u32);
flags!(ImageViewCreateFlags, []);

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

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct MetalSurfaceCreateFlagsEXT(u32);
flags!(MetalSurfaceCreateFlagsEXT, []);

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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
        SurfaceTransformFlagsKHR::from(*self).fmt(f)
    }
}
impl Default for SurfaceTransformKHR {
    fn default() -> Self {
        Self::IDENTITY
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SurfaceTransformFlagsKHR(u32);
impl SurfaceTransformFlagsKHR {
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
flags!(
    SurfaceTransformFlagsKHR,
    [
        IDENTITY,
        ROTATE_90,
        ROTATE_180,
        ROTATE_270,
        HORIZONTAL_MIRROR,
        HORIZONTAL_MIRROR_ROTATE_90,
        HORIZONTAL_MIRROR_ROTATE_180,
        HORIZONTAL_MIRROR_ROTATE_270,
        INHERIT
    ]
);
impl From<SurfaceTransformKHR> for SurfaceTransformFlagsKHR {
    fn from(bit: SurfaceTransformKHR) -> Self {
        Self(bit.0)
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CompositeAlphaKHR(u32);
impl CompositeAlphaKHR {
    pub const OPAQUE: Self = Self(0x1);
    pub const PRE_MULTIPLIED: Self = Self(0x2);
    pub const POST_MULTIPLIED: Self = Self(0x4);
    pub const INHERIT: Self = Self(0x8);
}
impl std::fmt::Debug for CompositeAlphaKHR {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        CompositeAlphaFlagsKHR::from(*self).fmt(f)
    }
}
impl Default for CompositeAlphaKHR {
    fn default() -> Self {
        Self::OPAQUE
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CompositeAlphaFlagsKHR(u32);
impl CompositeAlphaFlagsKHR {
    pub const OPAQUE: Self = Self(0x1);
    pub const PRE_MULTIPLIED: Self = Self(0x2);
    pub const POST_MULTIPLIED: Self = Self(0x4);
    pub const INHERIT: Self = Self(0x8);
}
flags!(
    CompositeAlphaFlagsKHR,
    [OPAQUE, PRE_MULTIPLIED, POST_MULTIPLIED, INHERIT]
);
impl From<CompositeAlphaKHR> for CompositeAlphaFlagsKHR {
    fn from(bit: CompositeAlphaKHR) -> Self {
        Self(bit.0)
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageUsageFlags(u32);
impl ImageUsageFlags {
    pub const TRANSFER_SRC: Self = Self(0x01);
    pub const TRANSFER_DST: Self = Self(0x02);
    pub const SAMPLED: Self = Self(0x04);
    pub const STORAGE: Self = Self(0x08);
    pub const COLOR_ATTACHMENT: Self = Self(0x10);
    pub const DEPTH_STENCIL_ATTACHMENT: Self = Self(0x20);
    pub const TRANSIENT_ATTACHMENT: Self = Self(0x40);
    pub const INPUT_ATTACHMENT: Self = Self(0x80);
}
flags!(
    ImageUsageFlags,
    [
        TRANSFER_SRC,
        TRANSFER_DST,
        SAMPLED,
        STORAGE,
        COLOR_ATTACHMENT,
        DEPTH_STENCIL_ATTACHMENT,
        TRANSIENT_ATTACHMENT,
        INPUT_ATTACHMENT
    ]
);

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
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

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageAspectFlags(u32);
impl ImageAspectFlags {
    pub const COLOR: Self = Self(0x01);
    pub const DEPTH: Self = Self(0x02);
    pub const STENCIL: Self = Self(0x04);
    pub const METADATA: Self = Self(0x08);
    pub const PLANE_0: Self = Self(0x10);
    pub const PLANE_1: Self = Self(0x20);
    pub const PLANE_2: Self = Self(0x40);
    pub const NONE: Self = Self(0);
}
flags!(
    ImageAspectFlags,
    [COLOR, DEPTH, STENCIL, METADATA, PLANE_0, PLANE_1, PLANE_2]
);

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommandPoolCreateFlags(u32);
impl CommandPoolCreateFlags {
    pub const TRANSIENT: Self = Self(0x1);
    pub const RESET_COMMAND_BUFFER: Self = Self(0x2);
    pub const PROTECTED: Self = Self(0x4);
}
flags!(CommandPoolCreateFlags, [TRANSIENT, RESET_COMMAND_BUFFER, PROTECTED]);

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommandPoolResetFlags(u32);
impl CommandPoolResetFlags {
    pub const RELEASE_RESOURCES: Self = Self(0x1);
}
flags!(CommandPoolResetFlags, [RELEASE_RESOURCES]);

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CommandBufferLevel(u32);
impl CommandBufferLevel {
    pub const PRIMARY: Self = Self(0);
    pub const SECONDARY: Self = Self(1);
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommandBufferUsageFlags(u32);
impl CommandBufferUsageFlags {
    pub const ONE_TIME_SUBMIT: Self = Self(0x1);
    pub const RENDER_PASS_CONTINUE: Self = Self(0x2);
    pub const SIMULTANEOUS_USE: Self = Self(0x4);
}
flags!(
    CommandBufferUsageFlags,
    [ONE_TIME_SUBMIT, RENDER_PASS_CONTINUE, SIMULTANEOUS_USE]
);

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
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

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubpassDescriptionFlags(u32);
flags!(SubpassDescriptionFlags, []);

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct PipelineBindPoint(u32);
impl PipelineBindPoint {
    pub const GRAPHICS: Self = Self(0);
    pub const COMPUTE: Self = Self(1);
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct FramebufferCreateFlags(u32);
impl FramebufferCreateFlags {
    pub const IMAGELESS: Self = Self(0);
}
flags!(FramebufferCreateFlags, [IMAGELESS]);

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct RenderPassCreateFlags(u32);
flags!(RenderPassCreateFlags, []);

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct ColorSpaceKHR(u32);
impl ColorSpaceKHR {
    pub const SRGB_NONLINEAR_KHR: Self = Self(0);
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SwapchainCreateFlagsKHR(u32);
impl SwapchainCreateFlagsKHR {
    pub const SPLIT_INSTANCE_BIND_REGIONS: Self = Self(0x1);
    pub const PROTECTED: Self = Self(0x2);
    pub const MUTABLE_FORMAT: Self = Self(0x4);
}
flags!(
    SwapchainCreateFlagsKHR,
    [SPLIT_INSTANCE_BIND_REGIONS, PROTECTED, MUTABLE_FORMAT]
);

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct SharingMode(u32);
impl SharingMode {
    pub const EXCLUSIVE: Self = Self(0);
    pub const CONCURRENT: Self = Self(1);
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
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
