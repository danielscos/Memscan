// memory optimization for FLTK
//
use std::alloc::{GlobalAlloc, Layout, System};
use std::fmt::format;
use std::sync::atomic::{AtomicUsize, Ordering};

// custom alloc
pub struct TrackingAllocator;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe {
            let ret = System.alloc(layout);
            if !ret.is_null() {
                ALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
            }
            ret
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe {
            System.dealloc(ptr, layout);
            ALLOCATED.fetch_sub(layout.size(), Ordering::SeqCst);
        }
    }
}

#[global_allocator]
static GLOBAL: TrackingAllocator = TrackingAllocator;

pub fn get_allocated_bytes() -> usize {
    ALLOCATED.load(Ordering::SeqCst)
}

use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref MEMORY_USAGE: Mutex<usize> = Mutex::new(0);
}

pub fn track_allocation(size: usize) {
    if let Ok(mut usage) = MEMORY_USAGE.lock() {
        *usage += size;
    }
}

pub fn track_deallocation(size: usize) {
    if let Ok(mut usage) = MEMORY_USAGE.lock() {
        *usage = usage.saturating_sub(size);
    }
}

pub fn get_tracked_memory() -> usize {
    match MEMORY_USAGE.lock() {
        Ok(usage) => *usage,
        Err(_) => 0,
    }
}

// object pool
pub struct ObjectPool<T> {
    objects: Vec<T>,
    factory: fn() -> T,
}

impl<T> ObjectPool<T> {
    pub fn new(capacity: usize, factory: fn() -> T) -> Self {
        let mut objects = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            objects.push(factory());
        }
        Self { objects, factory }
    }

    pub fn get(&mut self) -> T {
        self.objects.pop().unwrap_or_else(|| ((self.factory)()))
    }

    pub fn put(&mut self, obj: T) {
        if self.objects.len() < self.objects.capacity() {
            self.objects.push(obj);
        }
    }
}

pub fn get_memory_stats() -> (usize, usize) {
    let allocated = get_allocated_bytes();
    let tracked = get_tracked_memory();
    (allocated, tracked)
}

pub fn format_bytes(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
