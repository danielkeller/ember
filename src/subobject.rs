// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::cell::UnsafeCell;
use std::ops::Deref;
use std::sync::{Arc, Weak};

/// Can be used to model subobject relationships that happen in C. An object
/// has a unique Owner, which can be used to access it mutably, multiple
/// Subobjects, which cannot access the object but prolong its lifetime, and
/// multiple WeakSubobjects, which can be upgrade()d to Subobjects.
pub struct Owner<T: ?Sized>(Arc<UnsafeCell<T>>);

pub struct Subobject<T: ?Sized>(Arc<UnsafeCell<T>>);

pub struct WeakSubobject<T: ?Sized>(Weak<UnsafeCell<T>>);

unsafe impl<T: Send> Send for Owner<T> {}
unsafe impl<T: Sync> Sync for Owner<T> {}
impl<T: std::panic::UnwindSafe> std::panic::UnwindSafe for Owner<T> {}
impl<T: std::panic::RefUnwindSafe> std::panic::RefUnwindSafe for Owner<T> {}

unsafe impl<T: Send> Send for Subobject<T> {}
unsafe impl<T> Sync for Subobject<T> {}
impl<T> std::panic::UnwindSafe for Subobject<T> {}
impl<T> std::panic::RefUnwindSafe for Subobject<T> {}

unsafe impl<T: Send> Send for WeakSubobject<T> {}
unsafe impl<T> Sync for WeakSubobject<T> {}
impl<T> std::panic::UnwindSafe for WeakSubobject<T> {}
impl<T> std::panic::RefUnwindSafe for WeakSubobject<T> {}

impl<T> Owner<T> {
    pub fn new(value: T) -> Self {
        Self(Arc::new(UnsafeCell::new(value)))
    }
    /// Fails if `arc` has other strong or weak refs.
    pub fn from_arc(mut arc: Arc<T>) -> Result<Self, Arc<T>> {
        if Arc::get_mut(&mut arc).is_none() {
            return Err(arc);
        }
        // Safety: UnsafeCell is repr(transparent)
        Ok(Self(unsafe { arc_transmute(arc) }))
    }
    pub fn into_arc(this: Self) -> Arc<T> {
        // Safety: UnsafeCell is repr(transparent)
        unsafe { arc_transmute(this.0) }
    }
    // pub fn downgrade(this: &Self) -> WeakSubobject<T> {
    //     WeakSubobject(Arc::downgrade(&this.0))
    // }
    // pub fn try_unwrap(this: Self) -> std::result::Result<T, Self> {
    //     Arc::try_unwrap(this.0).map(UnsafeCell::into_inner).map_err(Owner)
    // }
    pub fn ptr_eq(this: &Self, other: &Subobject<T>) -> bool {
        Arc::ptr_eq(&this.0, &other.0)
    }
}

impl<T> Subobject<T> {
    pub fn new(value: &Owner<T>) -> Subobject<T> {
        Subobject(value.0.clone())
    }
    // pub fn downgrade(&self) -> WeakSubobject<T> {
    //     WeakSubobject(Arc::downgrade(&self.0))
    // }
}

impl<T> Clone for Subobject<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Send + Sync + 'static> Subobject<T> {
    pub fn erase(self) -> Arc<dyn Send + Sync> {
        // Safety: UnsafeCell is repr(transparent)
        // Safety: You can't access the object through dyn Send + Sync
        unsafe { arc_transmute::<UnsafeCell<T>, T>(self.0) }
    }
}

impl<T> Clone for WeakSubobject<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> WeakSubobject<T> {
    // pub fn new() -> Self {
    //     Self(Weak::new())
    // }
    // pub fn upgrade(&self) -> Option<Subobject<T>> {
    //     self.0.upgrade().map(Subobject)
    // }
}

// Safety: Only Owner can access the data
impl<T> std::ops::Deref for Owner<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.get() }
    }
}
impl<T> std::ops::DerefMut for Owner<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0.get() }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Owner<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Owner").field(self.deref()).finish()
    }
}

impl<T> std::fmt::Debug for Subobject<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Subobject").field(&self.0).finish()
    }
}

impl<T> std::fmt::Debug for WeakSubobject<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("WeakSubobject").field(&self.0).finish()
    }
}

unsafe fn arc_transmute<T, U>(arc: Arc<T>) -> Arc<U> {
    Arc::from_raw(Arc::into_raw(arc) as *const U)
}
