extern crate libc;
use std::result;
use std::fmt;
use super::mbuf::*;
use super::interface::Result;
use super::interface::ZCSIError;
use super::interface::ConstFromU8;
use super::interface::MutFromU8;
use super::interface::EndOffset;
#[link(name = "zcsi")]
extern {
    fn mbuf_alloc_bulk(array: *mut *mut MBuf, len: u16, cnt: i32) -> i32;
    fn mbuf_free_bulk(array: *mut *mut MBuf, cnt: i32) -> i32;
}

fn alloc_packet_batch(batch: &mut PacketBatch, len: u16, cnt: i32) -> result::Result<(), ()> {
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

fn free_packet_batch(batch: &mut PacketBatch) -> result::Result<(), ()> {
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

// PacketBatch
pub struct PacketBatch {
    array: Vec<*mut MBuf>,
    cnt: i32,
    start: usize,
    end: usize
}

// FIXME: Ensure we are not exporting this
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

impl PacketBatch {
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
        let mut idx = self.start;
        while idx < self.end {
            let val = unsafe { &*self.array[idx] };
            println!("Buf address is {:p} {:p}", val.data_address(0), self.array[idx]);
            idx = idx + 1;
        }
    }

    pub fn dump<T: ConstFromU8 + fmt::Display>(&self) {
        let mut idx = self.start;
        while idx < self.end {
            let val = unsafe { &*self.array[idx] };
            println!("{}", T::from_u8(val.data_address(0)));
            idx += 1;
        }
    }

    pub fn dump_at_offset<T: ConstFromU8 + fmt::Display>(&self, offsets: &Vec<usize>) {
        let mut idx = self.start;
        let mut oidx = 0;
        while idx < self.end {
            let val = unsafe { &*self.array[idx] };
            println!("{}", T::from_u8(val.data_address(offsets[oidx])));
            idx += 1; oidx += 1;
        }
    }

    #[inline]
    pub fn transform<T: MutFromU8>(&mut self, transformer:&Fn(&mut T)) {
        let mut idx = self.start;
        while idx < self.end {
            let val = unsafe { &mut *self.array[idx] };
            transformer(T::from_u8(val.data_address(0)));
            idx += 1;
        }
    }

    #[inline]
    pub fn transform_at_offset<T: MutFromU8>(&mut self, offsets: &Vec<usize>, transformer:&Fn(&mut T)) {
        let mut oidx = 0;
        let mut idx = self.start;
        while idx < self.end {
            let val = unsafe { &mut *self.array[idx] };
            transformer(T::from_u8(val.data_address(offsets[oidx])));
            idx += 1; oidx += 1;
        }
    }

    #[inline]
    pub fn find_offsets<T: EndOffset + ConstFromU8>(&self) -> Vec<usize> {
        let mut offsets = Vec::<usize>::with_capacity((self.end - self.start) as usize);
        let mut idx = self.start;
        while idx < self.end {
            let val = unsafe { &*self.array[idx] };
            offsets.push(T::offset(T::from_u8(val.data_address(0))));
            idx += 1;
        }
        offsets
    }

    #[inline]
    pub fn offsets_efficient<T: EndOffset + ConstFromU8>(&self, offsets: &mut Vec<usize>) {
        let mut idx = self.start;
        while idx < self.end {
            let val = unsafe { &*self.array[idx] };
            offsets.push(T::offset(T::from_u8(val.data_address(0))));
            idx += 1;
        }
    }
}

impl Drop for PacketBatch {
    fn drop(&mut self) {
        let _ = free_packet_batch(self);
    }
}
