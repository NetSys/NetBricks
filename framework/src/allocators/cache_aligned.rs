use alloc::heap::{allocate, deallocate};
use std::mem::size_of;
use std::ops::{Deref, DerefMut};
use std::ptr::{self, Unique};

const CACHE_LINE_SIZE: usize = 64;
unsafe fn allocate_cache_line(size: usize) -> *mut u8 {
    allocate(size, CACHE_LINE_SIZE)
}

pub struct CacheWrapper<T> {
    ptr: Unique<T>,
}

impl<T> Drop for CacheWrapper<T> {
    fn drop(&mut self) {
        unsafe {
            deallocate(*self.ptr as *mut u8, size_of::<T>(), 64);
        }
    }
}

impl<T> Deref for CacheWrapper<T> {
    type Target = T;
    fn deref<'a>(&'a self) -> &'a T {
        unsafe { self.ptr.get() }
    }
}

impl<T> DerefMut for CacheWrapper<T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        unsafe { self.ptr.get_mut() }
    }
}

pub fn cache_allocate<T: Sized>(src: T) -> CacheWrapper<T> {
    unsafe {
        let alloc = allocate_cache_line(size_of::<T>()) as *mut T;
        ptr::write(alloc, src);
        CacheWrapper { ptr: Unique::new(alloc) }
    }
}
