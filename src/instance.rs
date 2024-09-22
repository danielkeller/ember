// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::load;
use crate::load::InstanceFn;
use crate::types::*;

struct Impl {
    handle: Handle<VkInstance>,
    fun: InstanceFn,
}

/// A driver instance.
#[derive(Clone)]
pub struct Instance {
    inner: Arc<Impl>,
}

impl std::fmt::Debug for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.handle.fmt(f)
    }
}

impl Drop for Impl {
    fn drop(&mut self) {
        unsafe { (self.fun.destroy_instance)(self.handle.borrow_mut(), None) }
    }
}

impl PartialEq for Instance {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}
impl Eq for Instance {}

impl Instance {
    /// Creates a new instance
    #[doc = crate::man_link!(vkCreateInstance)]
    pub fn new<'a>(info: &'a InstanceCreateInfo<'a>) -> Self {
        let mut handle = None;
        unsafe {
            (load::vk_create_instance())(info, None, &mut handle).unwrap()
        };
        let handle = handle.unwrap();
        let fun = InstanceFn::new(handle.borrow());
        Instance { inner: Arc::new(Impl { handle, fun }) }
    }

    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkInstance> {
        self.inner.handle.borrow()
    }

    /// Get the instance functions.
    pub fn fun(&self) -> &InstanceFn {
        &self.inner.fun
    }
}
