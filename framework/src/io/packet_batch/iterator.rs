use super::packet_batch::cast_from_u8;
use std::marker::PhantomData;
use super::super::interface::EndOffset;
/// An interface implemented by all batches for iterating through the set of packets in a batch.
/// This is one of two private interfaces
pub trait BatchIterator {
    fn start(&mut self) -> usize;
    unsafe fn next_address(&mut self, idx: usize) -> Option<(*mut u8, usize)>;
    unsafe fn next_payload(&mut self, idx: usize) -> Option<(*mut u8, usize)>;
    unsafe fn payload(&mut self, idx: usize) -> *mut u8;
    unsafe fn address(&mut self, idx: usize) -> *mut u8;
    unsafe fn base_address(&mut self, idx: usize) -> *mut u8;
    unsafe fn base_payload(&mut self, idx: usize) -> *mut u8;
    unsafe fn next_base_address(&mut self, idx: usize) -> Option<(*mut u8, usize)>;
    unsafe fn next_base_payload(&mut self, idx: usize) -> Option<(*mut u8, usize)>;
}

pub struct PacketBatchAddressIterator<'a, T> 
           where T: 'a + EndOffset {
    batch: &'a mut BatchIterator,
    idx: usize,
    phantom: PhantomData<T>,
}

impl<'a, T> PacketBatchAddressIterator<'a, T>
           where T: 'a + EndOffset {
    #[inline]
    pub fn new(batch: &mut BatchIterator) -> PacketBatchAddressIterator<T> {
        let start = batch.start();
        PacketBatchAddressIterator {
            batch: batch,
            idx: start,
            phantom: PhantomData,
        }
    }
}

impl<'a, T> Iterator for PacketBatchAddressIterator<'a, T>
           where T: 'a + EndOffset {
    type Item = &'a mut T;

    #[inline]
    fn next(&mut self) -> Option<&'a mut T> {
        let item = unsafe { self.batch.next_address(self.idx) };
        match item {
            Some((addr, idx)) => {
                let packet = cast_from_u8::<T>(addr);
                self.idx = idx;
                Some(packet)
            }
            None => None,
        }
    }
}
