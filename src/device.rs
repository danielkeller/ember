// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use crate::error::{Error, Result};
use crate::instance::Instance;
use crate::load::DeviceFn;
use crate::physical_device::PhysicalDevice;
use crate::queue::Queue;
use crate::types::*;

/// A logical device.
/// Note that references to the device and instance are subject to the
/// [drop check](https://doc.rust-lang.org/nomicon/dropck.html) and may not be
/// dangling during destruction of the referring object.
// TODO: According to the spec, other references generally are allowed to
// dangle during destruction. Do we respect this?
pub struct Device<'i> {
    handle: Handle<VkDevice>,
    pub(crate) fun: DeviceFn,
    physical_device: PhysicalDevice<'i>,
    limits: PhysicalDeviceLimits,
    enabled: PhysicalDeviceFeatures,
    memory_allocation_count: AtomicU32,
    sampler_allocation_count: AtomicU32,
    queues: Vec<u32>,
    queues_taken: AtomicBool,
    // Maybe include device_lost so we don't double panic all the time
}

impl std::fmt::Debug for Device<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.handle.fmt(f)
    }
}

impl PartialEq for Device<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}
impl Eq for Device<'_> {}
impl std::hash::Hash for Device<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.handle.hash(state)
    }
}

impl Drop for Device<'_> {
    fn drop(&mut self) {
        unsafe {
            (self.fun.device_wait_idle)(self.handle.borrow_mut()).unwrap();
            (self.fun.destroy_device)(self.handle.borrow_mut(), None);
        }
    }
}

impl<'i> Device<'i> {
    /// Create a logical device for this physical device. Queues are returned in
    /// the order requested in `info.queue_create_infos`.
    #[doc = crate::man_link!(vkCreateDevice)]
    pub fn new(
        phy: &PhysicalDevice<'i>, info: &DeviceCreateInfo<'_>,
    ) -> Result<Self> {
        let props = phy.queue_family_properties();
        let mut queues = vec![0; props.len()];
        for q in info.queue_create_infos {
            let i = q.queue_family_index as usize;
            if i >= props.len()
                || q.queue_priorities.len() > props[i].queue_count
            {
                return Err(Error::OutOfBounds);
            }
            queues[i] = q.queue_priorities.len();
        }

        let mut handle = None;
        unsafe {
            (phy.instance().fun.create_device)(
                phy.handle(),
                info,
                None,
                &mut handle,
            )?;
        }
        let handle = handle.unwrap();
        let fun = DeviceFn::new(phy.instance(), handle.borrow());
        Ok(Device {
            handle,
            fun,
            physical_device: phy.clone(),
            limits: phy.properties().limits,
            enabled: info.enabled_features.cloned().unwrap_or_default(),
            memory_allocation_count: 0.into(),
            sampler_allocation_count: 0.into(),
            queues,
            queues_taken: false.into(),
        })
    }

    /// Return the device's queues. Will panic if called more than once.
    pub fn take_queues(&self) -> Vec<Vec<Queue>> {
        if self.queues_taken.swap(true, Ordering::Relaxed) {
            panic!("Device::take_queues called more than once.");
        }
        self.queues
            .iter()
            .enumerate()
            .map(|(i, &n)| (0..n).map(|n| self.queue(i as u32, n)).collect())
            .collect()
    }
}

impl Device<'_> {
    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkDevice> {
        self.handle.borrow()
    }
    /// Returns the limits of the device.
    pub fn limits(&self) -> &PhysicalDeviceLimits {
        &self.limits
    }
    /// Returns the enabled features.
    pub fn enabled(&self) -> &PhysicalDeviceFeatures {
        &self.enabled
    }
    /// Returns the associated phyical device.
    pub fn physical_device(&self) -> &PhysicalDevice {
        &self.physical_device
    }
    /// Returns the associated instance.
    pub fn instance(&self) -> &Instance {
        self.physical_device.instance()
    }
    /// Returns true if a queue with this family index and index exists.
    pub fn has_queue(&self, queue_family_index: u32, queue_index: u32) -> bool {
        let i = queue_family_index as usize;
        i < self.queues.len() && self.queues[i] >= queue_index
    }
    pub(crate) fn increment_memory_alloc_count(&self) -> Result<()> {
        use std::sync::atomic::Ordering;
        // Reserve allocation number 'val'.
        // Overflow is incredibly unlikely here
        let val = self.memory_allocation_count.fetch_add(1, Ordering::Relaxed);
        if val >= self.limits.max_memory_allocation_count {
            self.memory_allocation_count.fetch_sub(1, Ordering::Relaxed);
            Err(Error::LimitExceeded)
        } else {
            Ok(())
        }
    }
    pub(crate) fn decrement_memory_alloc_count(&self) {
        use std::sync::atomic::Ordering;
        self.memory_allocation_count.fetch_sub(1, Ordering::Relaxed);
    }
    pub(crate) fn increment_sampler_alloc_count(&self) -> Result<()> {
        use std::sync::atomic::Ordering;
        // Reserve allocation number 'val'.
        // Overflow is incredibly unlikely here
        let val = self.sampler_allocation_count.fetch_add(1, Ordering::Relaxed);
        if val >= self.limits.max_sampler_allocation_count {
            self.sampler_allocation_count.fetch_sub(1, Ordering::Relaxed);
            Err(Error::LimitExceeded)
        } else {
            Ok(())
        }
    }
    pub(crate) fn decrement_sampler_alloc_count(&self) {
        use std::sync::atomic::Ordering;
        self.sampler_allocation_count.fetch_sub(1, Ordering::Relaxed);
    }
}
