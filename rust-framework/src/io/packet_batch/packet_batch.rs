use std::fmt;
use std::result;
use super::Act;
use super::internal_iface::ProcessPacketBatch;
use super::ParsedBatch;
use super::super::mbuf::*;
use super::super::interface::Result;
use super::super::interface::ZCSIError;
use super::super::interface::EndOffset;

// PacketBatch
pub struct PacketBatch {
    array: Vec<*mut MBuf>,
    cnt: i32,
    start: usize,
    end: usize
}

impl ProcessPacketBatch for PacketBatch {
    #[inline]
    fn start(&self) -> usize {
        self.start
    }

    #[inline]
    unsafe fn payload(&mut self, idx: usize) -> *mut u8 {
        let val = &mut *self.array[idx];
        val.data_address(0)
    }

    #[inline]
    unsafe fn address(&mut self, idx: usize) -> *mut u8 {
        let val = &mut *self.array[idx];
        val.data_address(0)
    }

    #[inline]
    unsafe fn next_address(&mut self, idx: usize) -> Option<(*mut u8, usize)> {
        if idx < self.end {
            Some((self.address(idx), idx + 1))
        } else {
            None
        }
    }

    #[inline]
    unsafe fn next_payload(&mut self, idx: usize) -> Option<(*mut u8, usize)> {
        if idx < self.end {
            Some((self.payload(idx), idx + 1))
        } else {
            None
        }
    }
}

impl Act for PacketBatch {
    #[inline]
    fn act(&mut self) -> &mut Self {
        self
    }
}

impl PacketBatch {
    #[inline]
    pub fn parse<T:EndOffset>(&mut self) -> ParsedBatch<T, Self> {
        ParsedBatch::<T, Self>::new(self)
    }

    pub fn new(cnt: i32) -> PacketBatch {
        PacketBatch { array: Vec::<*mut MBuf>::with_capacity(cnt as usize), cnt: cnt, start: 0, end: 0}
    }

    pub fn allocate_batch_with_size(&mut self, len: u16) -> Result<&mut Self> {
        let cnt = self.cnt;
        match alloc_packet_batch(self, len, cnt) {
            Ok(_) => Ok(self),
            Err(_) => Err(ZCSIError::FailedAllocation)
        }
    }

    pub fn allocate_partial_batch_with_size(&mut self, len: u16, cnt: i32) -> Result<&mut Self> {
        match alloc_packet_batch(self, len, cnt) {
            Ok(_) => Ok(self),
            Err(_) => Err(ZCSIError::FailedAllocation)
        }
    }

    pub fn deallocate_batch(&mut self) -> Result<&mut Self> {
        match free_packet_batch(self) {
            Ok(_) => Ok(self),
            Err(_) => Err(ZCSIError::FailedDeallocation)
        }
    }

    pub fn available(&self) -> usize {
        (self.end - self.start)
    }

    pub fn max_size(&self) -> i32 {
        self.cnt
    }

    pub fn dump_addr(&self) {
        let start = self.start;
        let end = self.end;
        for idx in start..end  {
            let val = unsafe { &*self.array[idx] };
            println!("Buf address is {:p} {:p}", val.data_address(0), self.array[idx]);
        }
    }

    pub fn dump<T: fmt::Display>(&self) {
        //let mut idx = self.start;
        let start = self.start;
        let end = self.end;
        for idx in start..end {
            let val = unsafe { &*self.array[idx] };
            println!("{}", cast_from_u8::<T>(val.data_address(0)));
        }
    }
}

impl Drop for PacketBatch {
    fn drop(&mut self) {
        let _ = free_packet_batch(self);
    }
}

// Some low level functions that need access to private members.
#[link(name = "zcsi")]
extern {
    fn mbuf_alloc_bulk(array: *mut *mut MBuf, len: u16, cnt: i32) -> i32;
    fn mbuf_free_bulk(array: *mut *mut MBuf, cnt: i32) -> i32;
}

pub fn alloc_packet_batch(batch: &mut PacketBatch, len: u16, cnt: i32) -> result::Result<(), ()> {
    unsafe {
        if batch.array.capacity() < (cnt as usize) {
            Err(())
        } else {
            let parray = batch.array.as_mut_ptr();
            let ret  = mbuf_alloc_bulk(parray, len, cnt);
            if ret == 0 {
                batch.start = 0;
                batch.end = cnt as usize;
                batch.array.set_len(batch.end);
                Ok(())
            } else {
                Err(())
            }
        }
    }
}

pub fn free_packet_batch(batch: &mut PacketBatch) -> result::Result<(), ()> {
    unsafe {
        if batch.end > batch.start {
            let parray = packet_ptr(batch);
            let ret = mbuf_free_bulk(parray, ((batch.end - batch.start) as i32));
            // If free fails, I am not sure we can do much to recover this batch.
            batch.end = 0;
            batch.start = 0;
            batch.array.set_len(batch.end);
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

/// Use this to get a pointer that can be used to call into native code.
#[inline]
pub unsafe fn packet_ptr(batch: &mut PacketBatch) -> *mut *mut MBuf {
    batch.array.as_mut_ptr().offset(batch.start as isize)
}

#[inline]
pub unsafe fn consumed_batch(batch: &mut PacketBatch, consumed: usize) {
    batch.start += consumed;
    if batch.start == batch.end {
        batch.start = 0;
        batch.end = 0;
        batch.array.set_len(batch.end);
    }
}

#[inline]
pub unsafe fn add_to_batch(batch: &mut PacketBatch, added: usize) {
    assert_eq!(batch.start, 0);
    batch.start = 0;
    batch.end = added;
    batch.array.set_len(batch.end);
}

#[inline]
pub fn cast_from_u8<'a, T:'a>(data: *mut u8) -> &'a mut T {
    let typecast = data as *mut T;
    unsafe {&mut *typecast}
}
