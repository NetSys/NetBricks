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
        let data_len = (*mbuf).data_len();
        if len > 0 && offset <= data_len {
            let copy_len = data_len - offset;
            if (*mbuf).add_data_end(len) > 0 {
                if copy_len > 0 {
                    let src = (*mbuf).data_address(offset);
                    let dst = (*mbuf).data_address(offset + len);
                    std::ptr::copy(src, dst, copy_len);
                }
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
            let data_len = (*mbuf).data_len();
            let src_offset = offset + len;
            if src_offset < data_len {
                let src = (*mbuf).data_address(offset + len);
                let dst = (*mbuf).data_address(offset);
                std::ptr::copy(src, dst, data_len - src_offset);
                (*mbuf).remove_data_end(len);
            } else if src_offset == data_len {
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

/// Trims the buffer to the length specified
#[inline]
pub fn trim(mbuf: *mut MBuf, to_len: usize) -> Result<()> {
    unsafe {
        let data_len = (*mbuf).data_len();
        if data_len > to_len {
            (*mbuf).remove_data_end(data_len - to_len);
            Ok(())
        } else {
            return Err(BufferError::NotResized.into());
        }
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

    pub const BUFFER: [u8; 16] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

    #[test]
    fn alloc_buffer_tail() {
        dpdk_test! {
            unsafe {
                let mbuf = mbuf_alloc();
                assert!(alloc(mbuf, 0, 16).is_ok());
                assert_eq!(16, (*mbuf).data_len());

                let _ = write_slice(mbuf, 0, &BUFFER);

                assert!(alloc(mbuf, 16, 8).is_ok());
                assert_eq!(24, (*mbuf).data_len());
                let slice = &(*read_slice::<u8>(mbuf, 0, 24).unwrap());

                // data untouched
                assert_eq!(BUFFER, slice[..16]);
            }
        }
    }

    #[test]
    fn alloc_buffer_middle() {
        dpdk_test! {
            unsafe {
                let mbuf = mbuf_alloc();
                let _ = alloc(mbuf, 0, 16);
                let _ = write_slice(mbuf, 0, &BUFFER);

                // alloc in the middle
                assert!(alloc(mbuf, 4, 8).is_ok());
                assert_eq!(24, (*mbuf).data_len());
                let slice = &(*read_slice::<u8>(mbuf, 0, 24).unwrap());

                // [0..4] untouched
                assert_eq!(BUFFER[..4], slice[..4]);
                // [4..12] untouched, this is the 'new' memory
                assert_eq!(BUFFER[4..12], slice[4..12]);
                // copied [4..16] to [12..24]
                assert_eq!(BUFFER[4..], slice[12..24]);
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
    fn dealloc_buffer_tail() {
        dpdk_test! {
            unsafe {
                let mbuf = mbuf_alloc();
                let _ = alloc(mbuf, 0, 16);
                let _ = write_slice(mbuf, 0, &BUFFER);

                assert!(dealloc(mbuf, 8, 8).is_ok());
                assert_eq!(8, (*mbuf).data_len());
                let slice = &(*read_slice::<u8>(mbuf, 0, 8).unwrap());

                assert_eq!(BUFFER[..8], slice[..8]);
            }
        }
    }

    #[test]
    fn dealloc_buffer_middle() {
        dpdk_test! {
            unsafe {
                let mbuf = mbuf_alloc();
                let _ = alloc(mbuf, 0, 16);
                let _ = write_slice(mbuf, 0, &BUFFER);

                assert!(dealloc(mbuf, 4, 8).is_ok());
                assert_eq!(8, (*mbuf).data_len());
                let slice = &(*read_slice::<u8>(mbuf, 0, 8).unwrap());

                // removed [4..12]
                assert_eq!(BUFFER[..4], slice[..4]);
                assert_eq!(BUFFER[12..], slice[4..]);
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

    #[test]
    fn trim_buffer() {
        dpdk_test! {
            unsafe {
                let mbuf = mbuf_alloc();
                let _ = alloc(mbuf, 0, 16);
                let _ = write_slice(mbuf, 0, &BUFFER);

                assert!(trim(mbuf, 8).is_ok());
                assert_eq!(8, (*mbuf).data_len());
                let slice = &(*read_slice::<u8>(mbuf, 0, 8).unwrap());

                assert_eq!(BUFFER[..8], slice[..8]);
            }
        }
    }

    #[test]
    fn read_write_item() {
        dpdk_test! {
            unsafe {
                let mbuf = mbuf_alloc();
                let _ = alloc(mbuf, 0, 20);
                let _ = write_item::<[u8;16]>(mbuf, 0, &BUFFER);
                let item = read_item::<[u8;16]>(mbuf, 0).unwrap();
                assert_eq!(&BUFFER, &(*item));

                // read from the wrong offset should return junk
                let item = read_item::<[u8;16]>(mbuf, 2).unwrap();
                assert!(&BUFFER != &(*item));

                // read exceeds buffer should err
                assert!(read_item::<[u8;16]>(mbuf, 10).is_err());
            }
        }
    }

    #[test]
    fn read_write_slice() {
        dpdk_test! {
            unsafe {
                let mbuf = mbuf_alloc();
                let _ = alloc(mbuf, 0, 20);
                let _ = write_slice(mbuf, 0, &BUFFER);
                let slice = read_slice::<u8>(mbuf, 0, 16).unwrap();
                assert_eq!(&BUFFER, &(*slice));

                // read from the wrong offset should return junk
                let slice = read_slice::<u8>(mbuf, 2, 16).unwrap();
                assert!(&BUFFER != &(*slice));

                // read exceeds buffer should err
                assert!(read_slice::<u8>(mbuf, 10, 16).is_err());
            }
        }
    }
}
