use common::*;
use std::cmp::min;
use std::io::{Read, Write};

/// A ring buffer which can be used to insert and read ordered data.
pub struct RingBuffer {
    /// Head, signifies where a consumer should read from.
    head: usize,
    /// Tail, signifies where a producer should write.
    tail: usize,
    /// Size of the ring buffer.
    size: usize,
    /// Mask used for bit-wise wrapping operations.
    mask: usize,
    /// A Vec that holds this RingBuffer's data.
    vec: Vec<u8>,
}

unsafe impl Send for RingBuffer {}

#[cfg_attr(feature = "dev", allow(len_without_is_empty))]
impl RingBuffer {
    /// Create a new wrapping ring buffer. The ring buffer size is specified in page size (4KB) and must be a power of
    /// 2.
    pub fn new(pages: usize) -> Result<RingBuffer> {
        if pages & (pages - 1) != 0 {
            // We need pages to be a power of 2.
            return Err(ErrorKind::InvalidRingSize(pages).into());
        }

        let bytes = pages << 12;

        Ok(RingBuffer {
            head: 0,
            tail: 0,
            size: bytes,
            mask: bytes - 1,
            vec: vec![0; bytes],
        })
    }

    /// Reads data from self.vec, wrapping around the end of the Vec if necessary. Returns the
    /// number of bytes written.
    fn wrapped_read (&mut self, offset: usize, data: &mut [u8]) -> usize {
        let mut bytes: usize = 0;
        assert!(offset < self.size);
        assert!(data.len() <= self.size);

        bytes += (&self.vec[offset..]).read(data).unwrap();
        if offset + data.len() > self.size {
            let remaining = data.len() - bytes;
            bytes += (&self.vec[..remaining]).read(&mut data[bytes..]).unwrap();
        }
        bytes
    }

    /// Writes data to self.vec[offset:], wrapping around the end of the Vec if necessary. Returns
    /// the number of bytes written.
    fn wrapped_write(&mut self, offset: usize, data: &[u8]) -> usize {
        let mut bytes: usize = 0;
        assert!(offset < self.size);
        assert!(data.len() <= self.size);

        bytes += (&mut self.vec[offset..]).write(data).unwrap();
        if offset + data.len() > self.size {
            let remaining = data.len() - bytes;
            bytes += (&mut self.vec[..remaining]).write(&data[bytes..]).unwrap();
        }
        bytes
    }

    /// Write data at an offset of the buffer. Do not use this function if you use `write_at_tail`/`read_from_head`.
    #[inline]
    pub fn write_at_offset(&mut self, offset: usize, data: &[u8]) -> usize {
        self.wrapped_write(offset, data)
    }

    /// Read data from offset of the buffer. Do not use if using `write_at_tail`/`read_from_head`
    #[inline]
    pub fn read_from_offset(&mut self, offset: usize, mut data: &mut [u8]) -> usize {
        self.wrapped_read(offset, data)
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
        self.wrapped_write(offset, &data[..write])
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
            self.wrapped_write(index, &data[..write])
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
        self.wrapped_read(offset, &mut data[..to_read])
    }

    /// Read from the buffer, incrementing the read head. Returns bytes read.
    #[inline]
    pub fn read_from_head(&mut self, mut data: &mut [u8]) -> usize {
        let len = data.len();
        self.read_from_head_with_increment(data, len)
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
