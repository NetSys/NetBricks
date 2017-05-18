use alloc::heap::{allocate, deallocate};
use std::fmt;
use std::mem::size_of;
use std::ops::{Deref, DerefMut};
use std::ptr::{self, Unique};

const CACHE_LINE_SIZE: usize = 64;
unsafe fn allocate_cache_line(size: usize) -> *mut u8 {
    allocate(size, CACHE_LINE_SIZE)
}

pub struct CacheAligned<T: Sized> {
    uniq: Unique<T>,
}

impl<T: Sized> Drop for CacheAligned<T> {
    fn drop(&mut self) {
        unsafe {
            deallocate(self.uniq.as_ptr() as *mut u8, size_of::<T>(), 64);
        }
    }
}

impl<T: Sized> Deref for CacheAligned<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { self.uniq.as_ref() }
    }
}

impl<T: Sized> DerefMut for CacheAligned<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.uniq.as_mut() }
    }
}

impl<T: Sized> CacheAligned<T> {
    pub fn allocate(src: T) -> CacheAligned<T> {
        unsafe {
            let alloc = allocate_cache_line(size_of::<T>()) as *mut T;
            ptr::write(alloc, src);
            CacheAligned { uniq: Unique::new(alloc) }
        }
    }
}

impl<T: Sized> Clone for CacheAligned<T>
    where T: Clone
{
    fn clone(&self) -> CacheAligned<T> {
        unsafe {
            let alloc = allocate_cache_line(size_of::<T>()) as *mut T;
            ptr::copy(self.uniq.as_ptr() as *const T, alloc, 1);
            CacheAligned { uniq: Unique::new(alloc) }
        }
    }
}

impl<T: Sized> fmt::Display for CacheAligned<T>
    where T: fmt::Display
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        T::fmt(&*self, f)
    }
}
