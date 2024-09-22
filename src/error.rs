// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt::Debug;

#[doc = crate::man_link!(VkResult)]
#[repr(transparent)]
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VkResult(pub i32);

impl VkResult {
    pub fn is_success(self) -> bool {
        self.0 >= 0
    }
    #[track_caller]
    pub fn unwrap(self) {
        if !self.is_success() {
            panic!("{self:?}")
        }
    }
    #[track_caller]
    pub fn unwrap_unless_panicking(self) {
        if std::thread::panicking() {
            return;
        }
        if !self.is_success() {
            panic!("{self:?}")
        }
    }

    pub const NOT_READY: Self = Self(1);
    pub const TIMEOUT: Self = Self(2);
    pub const SUBOPTIMAL_HKR: Self = Self(1000001003);
    pub const DEVICE_LOST: Self = Self(-4);
    pub const OUT_OF_DEVICE_MEMORY: Self = Self(-2);
    pub const OUT_OF_POOL_MEMORY: Self = Self(-1000069000);
    pub const MEMORY_MAP_FAILED: Self = Self(-5);
    pub const OUT_OF_DATE_KHR: Self = Self(-1000001004);
}

impl Debug for VkResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self.0 {
            1 => "NOT_READY",
            2 => "TIMEOUT",
            5 => "INCOMPLETE",
            -1 => "OUT_OF_HOST_MEMORY",
            -2 => "OUT_OF_DEVICE_MEMORY",
            -3 => "INITIALIZATION_FAILED",
            -4 => "DEVICE_LOST",
            -5 => "ERROR_MEMORY_MAP_FAILED",
            -7 => "EXTENSION_NOT_PRESENT",
            -8 => "FEATURE_NOT_PRESENT",
            -9 => "INCOMPATIBLE_DRIVER",
            -1000000000 => "SURFACE_LOST_KHR",
            -1000069000 => "OUT_OF_POOL_MEMORY",
            1000001003 => "SUBOPTIMAL_HKR",
            -1000001004 => "OUT_OF_DATE_KHR",
            -1000255000 => "FULL_SCREEN_EXCLUSIVE_MODE_LOST_EXT",
            other => return write!(f, "VkResult({other})"),
        };
        f.write_str(str)
    }
}

/// A VkResult other than success.
#[doc = crate::man_link!(VkResult)]
#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VkError(pub i32);

impl From<VkResult> for Result<(), VkError> {
    fn from(value: VkResult) -> Self {
        if value.is_success() {
            Ok(())
        } else {
            Err(VkError(value.0))
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct OutOfDeviceMemory;

impl std::fmt::Display for OutOfDeviceMemory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Out of device memory")
    }
}
impl std::error::Error for OutOfDeviceMemory {}

impl VkResult {
    #[track_caller]
    pub fn unwrap_or_oom(self) -> Result<(), OutOfDeviceMemory> {
        match self {
            Self::OUT_OF_DEVICE_MEMORY => Err(OutOfDeviceMemory),
            other => Ok(other.unwrap()),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct OutOfPoolMemory;

impl std::fmt::Display for OutOfPoolMemory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Out of pool memory")
    }
}
impl std::error::Error for OutOfPoolMemory {}

impl VkResult {
    #[track_caller]
    pub fn unwrap_or_oopm(self) -> Result<(), OutOfPoolMemory> {
        match self {
            Self::OUT_OF_POOL_MEMORY => Err(OutOfPoolMemory),
            other => Ok(other.unwrap()),
        }
    }
}

// #[derive(Debug, PartialEq, Eq, Clone, Copy)]
// #[non_exhaustive]
// /// An error either from Vulkan or Maia.
// #[doc = crate::man_link!(VkResult)]
// pub enum Error {
//     /// Unknown Vulkan error.
//     Other,
//     /// The arguments provided to the function were incorrect.
//     InvalidArgument,
//     /// The object is misconfigured for the requested operation.
//     InvalidState,
//     /// The given index was out of bounds of the object.
//     OutOfBounds,
//     /// A required operation has not yet completed.
//     SynchronizationError,
//     /// The arguments exceed the limits of the device.
//     LimitExceeded,
//     #[doc = crate::man_link!(VkResult)]
//     NotReady,
//     #[doc = crate::man_link!(VkResult)]
//     Timeout,
//     #[doc = crate::man_link!(VkResult)]
//     Incomplete,
//     #[doc = crate::man_link!(VkResult)]
//     OutOfHostMemory,
//     #[doc = crate::man_link!(VkResult)]
//     OutOfDeviceMemory,
//     #[doc = crate::man_link!(VkResult)]
//     InitializationFailed,
//     #[doc = crate::man_link!(VkResult)]
//     ExtensionNotPresent,
//     #[doc = crate::man_link!(VkResult)]
//     FeatureNotPresent,
//     #[doc = crate::man_link!(VkResult)]
//     IncompatibleDriver,
//     #[doc = crate::man_link!(VkResult)]
//     DeviceLost,
//     #[doc = crate::man_link!(VkResult)]
//     SurfaceLostKHR,
//     #[doc = crate::man_link!(VkResult)]
//     OutOfPoolMemory,
//     #[doc = crate::man_link!(VkResult)]
//     SuboptimalHKR,
//     #[doc = crate::man_link!(VkResult)]
//     OutOfDateKHR,
//     #[doc = crate::man_link!(VkResult)]
//     FullScreenExclusiveModeLostEXT,
// }

// impl std::fmt::Display for Error {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         std::fmt::Debug::fmt(self, f)
//     }
// }
// impl std::error::Error for Error {}

// /// An error either from Vulkan or Maia.
// pub type Result<T> = std::result::Result<T, Error>;

// /// For functions that take an argument by value and need to return it in case
// /// of an error.
// pub struct ErrorAndSelf<T>(pub Error, pub T);

// impl<T> std::fmt::Debug for ErrorAndSelf<T> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         std::fmt::Debug::fmt(&self.0, f)
//     }
// }
// impl<T> std::fmt::Display for ErrorAndSelf<T> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         std::fmt::Debug::fmt(&self.0, f)
//     }
// }
// impl<T> From<ErrorAndSelf<T>> for Error {
//     fn from(ErrorAndSelf(err, _): ErrorAndSelf<T>) -> Self {
//         err
//     }
// }

// impl<T> std::error::Error for ErrorAndSelf<T> {}
// /// For functions that take an argument by value and need to return it in case
// /// of an error.
// pub type ResultAndSelf<T, S> = std::result::Result<T, ErrorAndSelf<S>>;
