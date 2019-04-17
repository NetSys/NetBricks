use common::Result;
use failure::Fail;
use native::zcsi::MBuf;
use packets::Fixed;
use std::slice;

/// Errors related to DPDK message buffer access
#[derive(Debug, Fail)]
pub enum BufferError {
    /// Failed to allocate a new buffer
    #[fail(display = "Failed to allocate a new buffer")]
    FailAlloc,

    /// The offset is larger than the buffer length
    #[fail(display = "Attempt to access bad packet offset {}", _0)]
    BadOffset(usize),

    /// Failed to resize the buffer
    #[fail(display = "Failed to resize the buffer")]
    NotResized,

    /// The struct is larger than the remaining buffer length
    #[fail(display = "Remaining buffer length less than struct size {}", _0)]
    OutOfBuffer(usize),
}

/// Returns a mutable pointer to `T` at buffer offset
#[inline]
pub fn read_item<T: Fixed>(mbuf: *mut MBuf, offset: usize) -> Result<*mut T> {
    unsafe {
        if offset > (*mbuf).data_len() {
            Err(BufferError::BadOffset(offset).into())
        } else if offset + T::size() > (*mbuf).data_len() {
            Err(BufferError::OutOfBuffer(T::size()).into())
        } else {
            Ok((*mbuf).data_address(offset) as *mut T)
        }
    }
}

/// Returns a mutable pointer to a slice of `T` of length `len` at buffer offset
#[inline]
pub fn read_slice<T: Fixed>(mbuf: *mut MBuf, offset: usize, len: usize) -> Result<*mut [T]> {
    unsafe {
        if offset > (*mbuf).data_len() {
            Err(BufferError::BadOffset(offset).into())
        } else if offset + T::size() * len > (*mbuf).data_len() {
            Err(BufferError::OutOfBuffer(T::size() * len).into())
        } else {
            let item0 = (*mbuf).data_address(offset) as *mut T;
            Ok(slice::from_raw_parts_mut(item0, len) as *mut [T])
        }
    }
}

/// Allocates buffer memory at offset by shifting it down by `len` bytes
#[inline]
pub fn alloc(mbuf: *mut MBuf, offset: usize, len: usize) -> Result<()> {
    unsafe {
        if len > 0 {
            if (*mbuf).add_data_end(len) > 0 {
                let src = (*mbuf).data_address(offset);
                let dst = (*mbuf).data_address(offset + len);
                std::ptr::copy(src, dst, len);
            } else {
                return Err(BufferError::NotResized.into());
            }
        }

        Ok(())
    }
}

/// Deallocates buffer memory at offset by shifting it up by `len` bytes
#[inline]
pub fn dealloc(mbuf: *mut MBuf, offset: usize, len: usize) -> Result<()> {
    unsafe {
        if len > 0 {
            if offset + len <= (*mbuf).data_len() {
                let src = (*mbuf).data_address(offset + len);
                let dst = (*mbuf).data_address(offset);
                std::ptr::copy(src, dst, len);
                (*mbuf).remove_data_end(len);
            } else {
                return Err(BufferError::NotResized.into());
            }
        }

        Ok(())
    }
}

/// Reallocates buffer memory
#[inline]
pub fn realloc(mbuf: *mut MBuf, offset: usize, len: isize) -> Result<()> {
    if len > 0 {
        alloc(mbuf, offset, len as usize)
    } else if len < 0 {
        dealloc(mbuf, offset, -len as usize)
    } else {
        Ok(())
    }
}

/// Writes `T` to buffer at offset and returns a mutable reference
#[inline]
pub fn write_item<T: Fixed>(mbuf: *mut MBuf, offset: usize, item: &T) -> Result<*mut T> {
    unsafe {
        if (*mbuf).data_len() >= offset + T::size() {
            let src = item as (*const T);
            let dst = (*mbuf).data_address(offset) as (*mut T);
            std::ptr::copy_nonoverlapping(src, dst, 1);
            read_item::<T>(mbuf, offset)
        } else {
            Err(BufferError::BadOffset(offset).into())
        }
    }
}

/// Writes slice of `T` to buffer at offset and returns a mutable reference
#[inline]
pub fn write_slice<T: Fixed>(mbuf: *mut MBuf, offset: usize, slice: &[T]) -> Result<*mut [T]> {
    unsafe {
        if (*mbuf).data_len() >= offset + (T::size() * slice.len()) {
            let src = slice.as_ptr();
            let dst = (*mbuf).data_address(offset) as (*mut T);
            std::ptr::copy_nonoverlapping(src, dst, slice.len());
            read_slice::<T>(mbuf, offset, slice.len())
        } else {
            Err(BufferError::BadOffset(offset).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dpdk_test;
    use native::zcsi::mbuf_alloc;

    #[test]
    fn alloc_buffer() {
        dpdk_test! {
            unsafe {
                let mbuf = mbuf_alloc();
                assert!(alloc(mbuf, 0, 200).is_ok());
                assert_eq!(200, (*mbuf).data_len());
            }
        }
    }

    #[test]
    fn alloc_too_much() {
        dpdk_test! {
            unsafe {
                let mbuf = mbuf_alloc();
                assert!(alloc(mbuf, 0, 999999).is_err());
            }
        }
    }

    #[test]
    fn dealloc_buffer() {
        dpdk_test! {
            unsafe {
                let mbuf = mbuf_alloc();
                assert!(alloc(mbuf, 0, 200).is_ok());
                assert!(dealloc(mbuf, 50, 100).is_ok());
                assert_eq!(100, (*mbuf).data_len());
            }
        }
    }

    #[test]
    fn dealloc_too_much() {
        dpdk_test! {
            unsafe {
                let mbuf = mbuf_alloc();
                assert!(alloc(mbuf, 0, 200).is_ok());
                assert!(dealloc(mbuf, 150, 100).is_err());
            }
        }
    }
}
