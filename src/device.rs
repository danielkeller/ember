use std::ffi::c_void;
use std::ptr::NonNull;

use crate::error::{Error, Result};
use crate::instance::Instance;
use crate::load::DeviceFn;
use crate::queue::Queue;
use crate::types::*;

pub struct Device {
    handle: DeviceRef<'static>,
    pub(crate) fun: DeviceFn,
    #[allow(dead_code)]
    instance: Arc<Instance>,
    queues: Vec<u32>,
}

impl std::fmt::Debug for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.handle.fmt(f)
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe { (self.fun.destroy_device)(self.handle, None) }
    }
}

impl Device {
    pub(crate) fn new(
        handle: DeviceRef<'static>,
        instance: Arc<Instance>,
        queues: Vec<u32>,
    ) -> Arc<Self> {
        Arc::new(Device {
            handle,
            fun: DeviceFn::new(&instance, handle),
            instance,
            queues,
        })
    }
    pub fn dev_ref(&self) -> DeviceRef<'_> {
        self.handle
    }
}

impl Device {
    /// Load device function. Panics if the string is not null-terminated or the
    /// function was not found.
    pub fn get_proc_addr(&self, name: &str) -> NonNull<c_void> {
        self.instance.load(self.dev_ref(), name)
    }

    pub fn queue(
        self: &Arc<Self>,
        family_index: u32,
        queue_index: u32,
    ) -> Result<Queue> {
        let i = family_index as usize;
        if i > self.queues.len() || self.queues[i] <= queue_index {
            return Err(Error::InvalidArgument);
        }
        let mut handle = None;
        unsafe {
            (self.fun.get_device_queue)(
                self.dev_ref(),
                family_index,
                queue_index,
                &mut handle,
            );
        }
        Ok(Queue::new(handle.unwrap(), self.clone()))
    }
}
