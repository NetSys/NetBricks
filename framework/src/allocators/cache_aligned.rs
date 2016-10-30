use alloc::heap::{allocate, deallocate};
use std::mem::size_of;
use std::ops::{Deref, DerefMut};

const CACHE_LINE_SIZE:usize = 64;
unsafe fn allocate_cache_line(size: usize) -> *mut u8 {
    allocate(size, CACHE_LINE_SIZE)
}

pub struct CacheWrapper<T> {
    ptr: *mut T
}

impl<T> Drop for CacheWrapper<T> {
    fn drop(&mut self) {
        unsafe { deallocate(self.ptr as *mut u8, size_of::<T>(), 64); }
    }
}

impl<T> Deref for CacheWrapper<T> {
    type Target = T;
    fn deref<'a>(&'a self) -> &'a T {
        unsafe {
            self.ptr.as_ref().unwrap()
        }
    }
}

impl<T> DerefMut for CacheWrapper<T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        unsafe { self.ptr.as_mut().unwrap() }
    }

}

pub fn cache_allocate<T: Sized>() -> CacheWrapper<T> {
    unsafe {
        CacheWrapper {
            ptr: allocate_cache_line(size_of::<T>()) as *mut T
        }
    }
}
