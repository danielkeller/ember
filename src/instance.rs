// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::error::Result;
use crate::load;
use crate::load::InstanceFn;
use crate::types::*;

/// A driver instance.
pub struct Instance {
    handle: Handle<VkInstance>,
    pub(crate) fun: InstanceFn,
}

impl std::fmt::Debug for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.handle.fmt(f)
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe { (self.fun.destroy_instance)(self.handle.handle_mut(), None) }
    }
}

impl PartialEq for Instance {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}
impl Eq for Instance {}

impl Instance {
    /// Creates a new instance
    #[doc = crate::man_link!(vkCreateInstance)]
    pub fn new<'a>(info: &'a InstanceCreateInfo<'a>) -> Result<Self> {
        let mut handle = None;
        unsafe { (load::vk_create_instance())(info, None, &mut handle)? };
        let handle = handle.unwrap();
        let fun = InstanceFn::new(handle.borrow());
        Ok(Instance { handle, fun })
    }
    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkInstance> {
        self.handle.borrow()
    }
}
