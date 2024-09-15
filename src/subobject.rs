// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::cell::UnsafeCell;
use std::ops::Deref;
use std::sync::Arc;

/// Can be used to model subobject relationships that happen in C. An object
/// has a unique Owner, which can be used to access it mutably, and multiple
/// Subobjects, which cannot access the object but prolong its lifetime.
pub struct Owner<T: ?Sized>(Arc<Subobject<T>>);

#[repr(transparent)]
pub struct Subobject<T: ?Sized>(UnsafeCell<T>);

unsafe impl<T: Sync> Sync for Owner<T> {}

unsafe impl<T: Send> Send for Subobject<T> {}
unsafe impl<T> Sync for Subobject<T> {}
impl<T> std::panic::UnwindSafe for Subobject<T> {}
impl<T> std::panic::RefUnwindSafe for Subobject<T> {}

impl<T> Owner<T> {
    pub fn new(value: T) -> Self {
        Self(Arc::new(Subobject(UnsafeCell::new(value))))
    }
    /// Fails if `arc` has other strong or weak refs.
    pub fn from_arc(mut arc: Arc<T>) -> Result<Self, Arc<T>> {
        if Arc::get_mut(&mut arc).is_none() {
            return Err(arc);
        }
        // Safety: Subobject is repr(transparent)
        Ok(Self(unsafe { arc_transmute(arc) }))
    }
    pub fn into_arc(this: Self) -> Arc<T> {
        // Safety: Subobject is repr(transparent)
        unsafe { arc_transmute(this.0) }
    }
    pub fn as_arc(&mut self) -> &mut Arc<Subobject<T>> {
        &mut self.0
    }
    pub fn downgrade(&self) -> Arc<Subobject<T>> {
        self.0.clone()
    }
}

// Safety: Only Owner can access the data
impl<T> std::ops::Deref for Owner<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 .0.get() }
    }
}
impl<T> std::ops::DerefMut for Owner<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 .0.get() }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Owner<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Owner").field(self.deref()).finish()
    }
}

impl<T> std::fmt::Debug for Subobject<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Subobject").finish()
    }
}

unsafe fn arc_transmute<T, U>(arc: Arc<T>) -> Arc<U> {
    Arc::from_raw(Arc::into_raw(arc) as *const U)
}
