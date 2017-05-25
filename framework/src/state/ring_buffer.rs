use common::*;
use libc::{self, c_void, ftruncate, mmap, munmap, shm_open, shm_unlink};
use std::cmp::min;
use std::ffi::CString;
use std::io;
use std::io::Write;
use std::ptr;
use std::slice;
use uuid::*;

/// A ring buffer which can be used to insert and read ordered data.
pub struct RingBuffer {
    /// Head, signifies where a consumer should read from.
    head: usize,
    /// Tail, signifies where a producer should write.
    tail: usize,
    /// Size of the ring buffer.
    size: usize,
    /// Used for computing circular things.
    mask: usize,
    /// Things for shm.
    bottom_map: *mut c_void,
    top_map: *mut c_void,
    /// rust buffer.
    buf: *mut u8,
}

unsafe impl Send for RingBuffer {}

impl Drop for RingBuffer {
    fn drop(&mut self) {
        unsafe {
            munmap(self.bottom_map, self.size);
            munmap(self.top_map, self.size);
        }
    }
}

#[cfg_attr(feature = "dev", allow(len_without_is_empty))]
impl RingBuffer {
    unsafe fn allocate(pages: usize) -> Result<RingBuffer> {
        if pages & (pages - 1) != 0 {
            // We need pages to be a power of 2.
            return Err(ErrorKind::InvalidRingSize(pages).into());
        }
        let bytes = pages << 12;
        let alloc_bytes = bytes * 2;

        // First open a SHM region.
        let name = CString::new(Uuid::new(UuidVersion::Random).unwrap().simple().to_string()).unwrap();
        let fd = shm_open(name.as_ptr(), libc::O_CREAT | libc::O_RDWR, 0o700);

        if fd < 0 {
            return Err(io::Error::last_os_error()).chain_err(|| ErrorKind::RingAllocationFailure);
        }

        if ftruncate(fd, bytes as i64) != 0 {
            libc::close(fd);
            shm_unlink(name.as_ptr());
            return Err(io::Error::last_os_error()).chain_err(|| ErrorKind::RingAllocationFailure);
        }

        // First get a big enough chunk of virtual memory. Fortunately for us this does not actually commit any physical
        // pages. We allocate twice as much memory so as to mirror the ring buffer.
        let address = mmap(ptr::null_mut(),
                           alloc_bytes,
                           libc::PROT_READ | libc::PROT_WRITE,
                           libc::MAP_ANONYMOUS | libc::MAP_SHARED,
                           fd,
                           0);
        if address == libc::MAP_FAILED {
            libc::close(fd);
            shm_unlink(name.as_ptr());
            return Err(io::Error::last_os_error()).chain_err(|| ErrorKind::RingAllocationFailure);
        };

        assert!((address as usize) % 4096 == 0);
        if shm_unlink(name.as_ptr()) != 0 {
            munmap(address, alloc_bytes);
            libc::close(fd);
            return Err(io::Error::last_os_error()).chain_err(|| ErrorKind::RingAllocationFailure);
        }

        let bottom = (address as *mut u8).offset(bytes as isize) as *mut libc::c_void;
        let bottom_address = mmap(bottom,
                                  bytes,
                                  libc::PROT_READ | libc::PROT_WRITE,
                                  libc::MAP_FIXED | libc::MAP_ANONYMOUS | libc::MAP_SHARED,
                                  fd,
                                  0);
        if bottom_address == libc::MAP_FAILED {
            munmap(address, alloc_bytes);
            libc::close(fd);
            return Err(io::Error::last_os_error()).chain_err(|| ErrorKind::RingAllocationFailure);
        }
        if bottom_address != bottom {
            munmap(address, bytes);
            munmap(bottom_address, bytes);
            return Err(ErrorKind::RingDuplicationFailure.into());
        }

        libc::close(fd);

        Ok(RingBuffer {
               head: 0,
               tail: 0,
               size: bytes,
               mask: bytes - 1,
               bottom_map: bottom,
               top_map: address,
               buf: address as *mut u8,
           })
    }

    /// Create a new wrapping ring buffer. The ring buffer size is specified in page size (4KB) and must be a power of
    /// 2. This only works on Linux, and can panic should any of the syscalls fail.
    pub fn new(pages: usize) -> Result<RingBuffer> {
        unsafe { RingBuffer::allocate(pages) }
    }

    /// Produce an immutable slice at an offset. The nice thing about our implementation is that we can produce slices
    /// despite using a circular ring buffer.
    #[inline]
    fn slice_at_offset(&self, offset: usize, len: usize) -> &[u8] {
        if len >= self.size {
            panic!("slice beyond buffer length");
        }
        unsafe {
            let begin = self.buf.offset(offset as isize);
            slice::from_raw_parts(begin, len)
        }
    }

    /// Produce a mutable slice.
    #[inline]
    fn mut_slice_at_offset(&self, offset: usize, len: usize) -> &mut [u8] {
        if len >= self.size {
            panic!("slice beyond buffer length");
        }
        unsafe {
            let begin = self.buf.offset(offset as isize);
            slice::from_raw_parts_mut(begin, len)
        }
    }

    /// Unsafe version of `mut_slice_at_offset` for use when writing to the tail of the ring buffer.
    #[inline]
    fn unsafe_mut_slice_at_offset(&self, offset: usize, len: usize) -> &mut [u8] {
        unsafe {
            let begin = self.buf.offset(offset as isize);
            slice::from_raw_parts_mut(begin, len)
        }
    }

    /// Unsafe version of `slice_at_offset` for use when reading from head of the ring buffer.
    #[inline]
    fn unsafe_slice_at_offset(&self, offset: usize, len: usize) -> &mut [u8] {
        unsafe {
            let begin = self.buf.offset(offset as isize);
            slice::from_raw_parts_mut(begin, len)
        }
    }

    /// Write data at an offset of the buffer. Do not use this function if you use `write_at_tail`/`read_from_head`.
    #[inline]
    pub fn write_at_offset(&mut self, offset: usize, data: &[u8]) -> usize {
        self.mut_slice_at_offset(offset, data.len())
            .write(data)
            .unwrap()
    }

    /// Read data from offset of the buffer. Do not use if using `write_at_tail`/`read_from_head`
    #[inline]
    pub fn read_from_offset(&mut self, offset: usize, mut data: &mut [u8]) -> usize {
        let write_size = min(data.len(), self.size);
        data.write(self.slice_at_offset(offset, write_size))
            .unwrap()
    }

    /// Write data at the end of the buffer. The amount of data written might be smaller than input.
    #[inline]
    pub fn write_at_tail(&mut self, data: &[u8]) -> usize {
        let available = self.mask.wrapping_add(self.head).wrapping_sub(self.tail);
        let write = min(data.len(), available);
        if write != data.len() {
            println!("Not writing all, available {}", available);
        }
        let offset = self.tail & self.mask;
        self.seek_tail(write);
        self.unsafe_mut_slice_at_offset(offset, write)
            .write(&data[0..write])
            .unwrap()
    }

    /// Write at an offset from the tail, useful when dealing with out-of-order data. Note, the caller is responsible
    /// for progressing the tail sufficiently (using `seek_tail`) when gaps are filled.
    #[inline]
    pub fn write_at_offset_from_tail(&mut self, offset: usize, data: &[u8]) -> usize {
        let available = self.mask.wrapping_add(self.head).wrapping_sub(self.tail);
        if available < offset {
            0 // The offset lies beyond where we can safely write.
        } else {
            let offset_tail = self.tail.wrapping_add(offset);
            let available_at_offset = self.mask.wrapping_add(self.head).wrapping_sub(offset_tail);
            let write = min(data.len(), available_at_offset);
            let index = offset_tail & self.mask;
            self.unsafe_mut_slice_at_offset(index, write)
                .write(&data[0..write])
                .unwrap()
        }
    }

    /// Data available to be read.
    #[inline]
    pub fn available(&self) -> usize {
        self.tail.wrapping_sub(self.head)
    }


    #[inline]
    fn read_offset(&self) -> usize {
        self.head & self.mask
    }

    /// Read from the buffer, incrementing the read head by `increment` bytes. Returns bytes read.
    #[inline]
    pub fn read_from_head_with_increment(&mut self, mut data: &mut [u8], increment: usize) -> usize {
        let offset = self.read_offset();
        let to_read = min(self.available(), data.len());
        self.head = self.head.wrapping_add(min(increment, to_read));
        (&mut data[0..to_read])
            .write(self.unsafe_slice_at_offset(offset, to_read))
            .unwrap()
    }

    /// Read from the buffer, incrementing the read head. Returns bytes read.
    #[inline]
    pub fn read_from_head(&mut self, mut data: &mut [u8]) -> usize {
        let len = data.len();
        self.read_from_head_with_increment(data, len)
    }

    /// Peek data from the read head. Note, that this slice is only valid until the next `read` or `write` operation.
    #[inline]
    pub fn peek_from_head(&self, len: usize) -> &[u8] {
        let offset = self.read_offset();
        let to_read = min(len, self.available());
        self.unsafe_slice_at_offset(offset, to_read)
    }

    /// Seek the read head by `seek` bytes (without actually reading any data). `seek` must be less-than-or-equal to the
    /// number of available bytes.
    #[inline]
    pub fn seek_head(&mut self, seek: usize) {
        let available = self.available();
        assert!(available >= seek, "Seek beyond available bytes.");
        self.head = self.head.wrapping_add(seek);
    }

    /// Length of the ring buffer.
    #[inline]
    pub fn len(&self) -> usize {
        self.size
    }

    /// In cases with out-of-order data this allows the write head (and hence the amount of available data) to be
    /// progressed without writing anything.
    #[inline]
    pub fn seek_tail(&mut self, increment_by: usize) {
        self.tail = self.tail.wrapping_add(increment_by);
    }

    #[inline]
    pub fn clear(&mut self) {
        self.head = 0;
        self.tail = 0;
    }
}
