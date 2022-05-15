use std::cell::UnsafeCell;
use std::ops::Deref;
use std::sync::{Arc, Weak};

/// Can be used to model subobject relationships that happen in C. An object
/// has a unique Owner, which can be used to access it mutably, multiple
/// Subobjects, which cannot access the object but prolong its lifetime, and
/// multiple WeakSubobjects, which can be upgrade()d to Subobjects.
pub struct Owner<T: ?Sized>(Arc<UnsafeCell<T>>);

#[derive(Clone, Debug)]
pub struct Subobject<T: ?Sized>(Arc<UnsafeCell<T>>);

#[derive(Clone, Debug)]
pub struct WeakSubobject<T: ?Sized>(Weak<UnsafeCell<T>>);

unsafe impl<T: Send> Send for Owner<T> {}
unsafe impl<T: Sync> Sync for Owner<T> {}

unsafe impl<T: Send> Send for Subobject<T> {}
unsafe impl<T> Sync for Subobject<T> {}

unsafe impl<T: Send> Send for WeakSubobject<T> {}
unsafe impl<T> Sync for WeakSubobject<T> {}

impl<T> Owner<T> {
    pub fn new(value: T) -> Self {
        Self(Arc::new(UnsafeCell::new(value)))
    }
    pub fn downgrade(this: &Self) -> WeakSubobject<T> {
        WeakSubobject(Arc::downgrade(&this.0))
    }
    pub fn try_unwrap(this: Self) -> std::result::Result<T, Self> {
        Arc::try_unwrap(this.0).map(UnsafeCell::into_inner).map_err(Owner)
    }
}

impl<T> Subobject<T> {
    pub fn new(value: &Owner<T>) -> Subobject<T> {
        Subobject(value.0.clone())
    }
    pub fn downgrade(this: &Self) -> WeakSubobject<T> {
        WeakSubobject(Arc::downgrade(&this.0))
    }
}

impl<T> WeakSubobject<T> {
    pub fn new() -> Self {
        Self(Weak::new())
    }
    pub fn upgrade(&self) -> Option<Subobject<T>> {
        self.0.upgrade().map(Subobject)
    }
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
