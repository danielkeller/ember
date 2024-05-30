// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[derive(Debug)]
pub struct CleanupQueue {
    cursor: usize,
    level: u64,
    array: Arc<Vec<QueueEntry>>,
}

#[derive(Debug)]
pub struct Cleanup {
    cursor: usize,
    level: u64,
    array: Weak<Vec<QueueEntry>>,
}

/// Cleans up on drop
#[derive(Debug)]
pub struct CleanupRAII(Cleanup);

struct QueueEntry {
    guard: AtomicU64,
    value: Cell<Option<Arc<dyn Send + Sync>>>,
}

impl std::fmt::Debug for QueueEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueueEntry")
            .field("guard", &self.guard)
            .finish_non_exhaustive()
    }
}

// Safety: Access to 'value' is guarded by 'guard'
unsafe impl Sync for QueueEntry {}
impl std::panic::UnwindSafe for QueueEntry {}
impl std::panic::RefUnwindSafe for QueueEntry {}

impl Default for QueueEntry {
    fn default() -> Self {
        Self { guard: AtomicU64::new(u64::MAX), value: Cell::new(None) }
    }
}

impl<T: Send + Sync + 'static> std::iter::Extend<StdArc<T>> for CleanupQueue {
    fn extend<I: IntoIterator<Item = StdArc<T>>>(&mut self, iter: I) {
        for elem in iter {
            self.push(elem)
        }
    }
}

impl CleanupQueue {
    fn new_array(capacity: usize) -> Arc<Vec<QueueEntry>> {
        let mut vec = Vec::new();
        vec.resize_with(capacity.max(1), Default::default);
        Arc::new(vec)
    }

    pub fn new(capacity: usize) -> Self {
        Self { cursor: 0, level: 0, array: Self::new_array(capacity) }
    }
    #[cfg(loom)]
    pub fn push(&mut self, value: StdArc<dyn Send + Sync>) {
        self.push_impl(Arc::from_std(value))
    }
    #[cfg(not(loom))]
    pub fn push(&mut self, value: Arc<dyn Send + Sync>) {
        self.push_impl(value)
    }
    fn push_impl(&mut self, value: Arc<dyn Send + Sync>) {
        let entry = &self.array[self.cursor];
        // No need to RMW, we're the only writer
        let guard = entry.guard.load(Ordering::Acquire);
        if guard == u64::MAX {
            // Safety: We're the only writer
            entry.value.set(Some(value));
            self.cursor = (self.cursor + 1) % self.array.len();
            entry.guard.store(self.level, Ordering::Release);
        } else {
            // Full
            let mut array = Self::new_array(self.array.len() * 2);
            std::mem::swap(&mut array, &mut self.array);
            self.cursor = 0;
            self.level = 0;
            self.push_impl(erase_arc(array));
            self.push_impl(value);
        }
    }
    /// Create a new object which can clean up everything previously pushed.
    pub fn new_cleanup(&mut self) -> Cleanup {
        let cursor = if self.cursor == 0 {
            self.array.len() - 1
        } else {
            self.cursor - 1
        };
        let result = Cleanup {
            cursor,
            level: self.level,
            array: downgrade(&self.array),
        };
        self.level += 1;
        result
    }
    /// Prevent any resources from being freed when the CleanupQueue is dropped.
    /// cleanup() can still be called on any outstanding Cleanup, freeing the
    /// corresponding resources.
    pub fn leak(&self) {
        std::mem::forget(self.array.clone())
    }
}

impl Cleanup {
    pub fn cleanup(&self) {
        let mut cursor = self.cursor;
        let array = match self.array.upgrade() {
            None => return, // Someone else got there first
            Some(array) => array,
        };
        loop {
            let entry = &array[cursor];
            let guard = entry.guard.load(Ordering::Relaxed);
            if guard > self.level {
                return; //Done
            }
            // Lock the location.
            if entry
                .guard
                .compare_exchange(
                    guard,
                    u64::MAX - 1,
                    Ordering::Acquire,
                    Ordering::Relaxed,
                )
                .is_err()
            {
                // We are racing another cleanup. Give up and let them do it.
                return;
            }
            // Safety: The value was written, we have exclusive access, and it
            // will not be overwritten until we set guard to u64::MAX.
            drop(entry.value.replace(None));
            entry.guard.store(u64::MAX, Ordering::Release);

            cursor = if cursor == 0 { array.len() - 1 } else { cursor - 1 };
        }
    }

    pub fn raii(self) -> CleanupRAII {
        CleanupRAII(self)
    }
}

impl Drop for CleanupRAII {
    fn drop(&mut self) {
        self.0.cleanup()
    }
}

#[cfg(loom)]
mod sync {
    pub use loom::{
        cell::Cell,
        sync::{
            atomic::{AtomicU64, Ordering},
            Arc,
        },
    };
    #[derive(Clone, Debug)]
    pub struct Weak<T>(Arc<T>);
    impl<T> Weak<T> {
        pub fn upgrade(&self) -> Option<Arc<T>> {
            Some(self.0.clone())
        }
    }
    pub fn downgrade<T>(arc: &Arc<T>) -> Weak<T> {
        Weak(arc.clone())
    }
    pub fn erase_arc(
        arc: Arc<impl Send + Sync + 'static>,
    ) -> Arc<dyn Send + Sync> {
        unsafe { Arc::from_raw(Arc::into_raw(arc) as *const _) }
    }
}
#[cfg(not(loom))]
mod sync {
    pub use std::{
        cell::Cell,
        sync::{
            atomic::{AtomicU64, Ordering},
            Arc, Weak,
        },
    };
    pub fn erase_arc(
        arc: Arc<impl Send + Sync + 'static>,
    ) -> Arc<dyn Send + Sync> {
        arc
    }
    pub fn downgrade<T>(arc: &Arc<T>) -> Weak<T> {
        Arc::downgrade(arc)
    }
}
use std::sync::Arc as StdArc;
use sync::*;

#[cfg(test_disabled)]
mod tests {
    use super::*;

    #[cfg(not(loom))]
    #[test]
    fn test_reordered_cleanup() {
        let mut queue = CleanupQueue::new(1);
        let mut arc = Arc::new(42);
        queue.push(arc.clone());
        let _cleanup1 = queue.new_cleanup(); // Points to old array
        queue.push(Arc::new(43));
        let cleanup2 = queue.new_cleanup(); // Points to new array
        assert!(Arc::get_mut(&mut arc).is_none());
        cleanup2.cleanup();
        assert!(Arc::get_mut(&mut arc).is_some());
    }

    #[cfg(loom)]
    #[test]
    fn test_cleanup_queue() {
        loom::model(|| {
            let mut queue = CleanupQueue::new(4);
            let arcs = || std::iter::from_fn(|| Some(StdArc::new(42))).take(3);
            queue.extend(arcs());
            let cleanup1 = queue.new_cleanup();
            queue.extend(arcs());
            let cleanup2 = queue.new_cleanup();
            loom::thread::spawn(move || {
                cleanup1.cleanup();
                cleanup2.cleanup();
            });
            queue.extend(arcs());
            let cleanup3 = queue.new_cleanup();
            queue.extend(arcs());
            let cleanup4 = queue.new_cleanup();
            loom::thread::spawn(move || {
                cleanup4.cleanup();
                cleanup3.cleanup();
            });
        })
    }
}
