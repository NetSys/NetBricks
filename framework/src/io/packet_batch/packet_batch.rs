use std::result;
use super::act::Act;
use super::Batch;
use super::iterator::BatchIterator;
use super::ReplaceBatch;
use super::TransformBatch;
use super::super::mbuf::*;
use super::super::interface::Result;
use super::super::interface::ZCSIError;
use super::super::interface::EndOffset;
use super::super::pmd::*;
use super::super::super::headers::*;

/// Base packet batch structure. This is the abstract structure on which all operations are built.
pub struct PacketBatch {
    array: Vec<*mut MBuf>,
    cnt: i32,
    start: usize,
    end: usize,
}

impl PacketBatch {
    // Public functions in PacketBatch are used internally within the framework.
    pub fn new(cnt: i32) -> PacketBatch {
        PacketBatch {
            array: Vec::<*mut MBuf>::with_capacity(cnt as usize),
            cnt: cnt,
            start: 0,
            end: 0,
        }
    }

    #[inline]
    pub fn allocate_batch_with_size(&mut self, len: u16) -> Result<&mut Self> {
        let cnt = self.cnt;
        match self.alloc_packet_batch(len, cnt) {
            Ok(_) => Ok(self),
            Err(_) => Err(ZCSIError::FailedAllocation),
        }
    }

    #[inline]
    pub fn allocate_partial_batch_with_size(&mut self, len: u16, cnt: i32) -> Result<&mut Self> {
        match self.alloc_packet_batch(len, cnt) {
            Ok(_) => Ok(self),
            Err(_) => Err(ZCSIError::FailedAllocation),
        }
    }

    #[inline]
    pub fn deallocate_batch(&mut self) -> Result<&mut Self> {
        match self.free_packet_batch() {
            Ok(_) => Ok(self),
            Err(_) => Err(ZCSIError::FailedDeallocation),
        }
    }

    #[inline]
    pub fn available(&self) -> usize {
        (self.end - self.start)
    }

    #[inline]
    pub fn max_size(&self) -> i32 {
        self.cnt
    }

    #[inline]
    pub fn recv_queue(&mut self, port: &mut PmdPort, queue: i32) -> Result<u32> {
        unsafe {
            match self.deallocate_batch() {
                Err(err) => Err(err),
                Ok(_) => self.recv_internal(port, queue),
            }
        }
    }

    // Some utility functions.

    #[inline]
    unsafe fn packet_ptr(&mut self) -> *mut *mut MBuf {
        self.array.as_mut_ptr().offset(self.start as isize)
    }

    #[inline]
    unsafe fn consumed_batch(&mut self, consumed: usize) {
        self.start += consumed;
        if self.start == self.end {
            self.start = 0;
            self.end = 0;
            self.array.set_len(self.end);
        }
    }

    #[inline]
    unsafe fn add_to_batch(&mut self, added: usize) {
        assert_eq!(self.start, 0);
        self.start = 0;
        self.end = added;
        self.array.set_len(self.end);
    }

    // Assumes we have already deallocated batch
    #[inline]
    unsafe fn recv_internal(&mut self, port: &mut PmdPort, queue: i32) -> Result<u32> {
        match port.recv_queue(queue, self.packet_ptr(), self.max_size() as i32) {
            e @ Err(_) => e,
            Ok(recv) => {
                self.add_to_batch(recv as usize);
                Ok(recv)
            }
        }
    }

    #[inline]
    fn alloc_packet_batch(&mut self, len: u16, cnt: i32) -> result::Result<(), ()> {
        unsafe {
            if self.array.capacity() < (cnt as usize) {
                Err(())
            } else {
                let parray = self.array.as_mut_ptr();
                let ret = mbuf_alloc_bulk(parray, len, cnt);
                if ret == 0 {
                    self.start = 0;
                    self.end = cnt as usize;
                    self.array.set_len(self.end);
                    Ok(())
                } else {
                    Err(())
                }
            }
        }
    }

    #[inline]
    fn free_packet_batch(&mut self) -> result::Result<(), ()> {
        unsafe {
            if self.end > self.start {
                let parray = self.packet_ptr();
                let ret = mbuf_free_bulk(parray, ((self.end - self.start) as i32));
                // If free fails, I am not sure we can do much to recover this self.
                self.end = 0;
                self.start = 0;
                self.array.set_len(self.end);
                if ret == 0 {
                    Ok(())
                } else {
                    Err(())
                }
            } else {
                Ok(())
            }
        }
    }
}

// A packet batch is also a batch (just a special kind)
impl BatchIterator for PacketBatch {
    /// The starting offset for packets in the current batch.
    #[inline]
    fn start(&mut self) -> usize {
        self.start
    }

    /// Return the payload for a given packet.
    ///
    /// idx must be a valid index.
    #[inline]
    unsafe fn payload(&mut self, idx: usize) -> *mut u8 {
        let val = &mut *self.array[idx];
        val.data_address(0)
    }

    /// Address for the payload for a given packet.
    #[inline]
    unsafe fn address(&mut self, idx: usize) -> *mut u8 {
        let val = &mut *self.array[idx];
        val.data_address(0)
    }

    /// Address for the next packet.
    #[inline]
    unsafe fn next_address(&mut self, idx: usize) -> Option<(*mut u8, usize)> {
        if idx < self.end {
            Some((self.address(idx), idx + 1))
        } else {
            None
        }
    }

    /// Payload for the next packet.
    #[inline]
    unsafe fn next_payload(&mut self, idx: usize) -> Option<(*mut u8, usize)> {
        if idx < self.end {
            Some((self.payload(idx), idx + 1))
        } else {
            None
        }
    }

    #[inline]
    unsafe fn base_address(&mut self, idx: usize) -> *mut u8 {
        self.address(idx)
    }

    #[inline]
    unsafe fn base_payload(&mut self, idx: usize) -> *mut u8 {
        self.payload(idx)
    }

    #[inline]
    unsafe fn next_base_address(&mut self, idx: usize) -> Option<(*mut u8, usize)> {
        self.next_address(idx)
    }

    #[inline]
    unsafe fn next_base_payload(&mut self, idx: usize) -> Option<(*mut u8, usize)> {
        self.next_payload(idx)
    }
}

/// Internal interface for packets.
impl Act for PacketBatch {
    #[inline]
    fn act(&mut self) -> &mut Self {
        self
    }

    #[inline]
    fn done(&mut self) -> &mut Self {
        self
    }

    #[inline]
    fn send_queue(&mut self, port: &mut PmdPort, queue: i32) -> Result<u32> {
        unsafe {
            port.send_queue(queue, self.packet_ptr(), self.available() as i32)
                .and_then(|sent| {
                    self.consumed_batch(sent as usize);
                    Ok(sent)
                })
        }
    }

    #[inline]
    fn capacity(&self) -> i32 {
        self.max_size()
    }
}

impl Batch for PacketBatch {
    type Parent = Self;
    type Header = NullHeader;
    #[inline]
    fn transform(self, _: &mut FnMut(&mut NullHeader)) -> TransformBatch<NullHeader, Self> {
        panic!("Cannot transform PacketBatch")
    }

    #[inline]
    fn replace(self, _: NullHeader) -> ReplaceBatch<NullHeader, Self> {
        panic!("Cannot replace PacketBatch")
    }

    #[inline]
    fn pop(&mut self) -> &mut Self {
        panic!("Cannot pop PacketBatch")
    }
}

impl Drop for PacketBatch {
    fn drop(&mut self) {
        let _ = self.free_packet_batch();
    }
}

#[inline]
pub fn cast_from_u8<'a, T: 'a>(data: *mut u8) -> &'a mut T {
    let typecast = data as *mut T;
    unsafe { &mut *typecast }
}

// Some low level functions that need access to private members.
#[link(name = "zcsi")]
extern "C" {
    fn mbuf_alloc_bulk(array: *mut *mut MBuf, len: u16, cnt: i32) -> i32;
    fn mbuf_free_bulk(array: *mut *mut MBuf, cnt: i32) -> i32;
}
