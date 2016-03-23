use super::iterator::{BatchIterator, PacketBatchAddressIterator};
use super::act::Act;
use super::Batch;
use super::HeaderOperations;
use super::super::interface::EndOffset;
use super::super::interface::Result;
use super::super::pmd::*;

pub struct TransformBatch<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator + Act
{
    parent: V,
    transformer: Box<FnMut(&mut T)>,
}

batch!{TransformBatch, [parent: V, transformer: Box<FnMut(&mut T)>], []}

impl<T, V> Act for TransformBatch<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator + Act
{
    fn act(&mut self) -> &mut Self {
        self.parent.act();
        {
            let ref mut f = self.transformer;
            let iter = PacketBatchAddressIterator::<T>::new(&mut self.parent);
            for packet in iter {
                f(packet);
            }
        }
        self
    }

    fn done(&mut self) -> &mut Self {
        self.parent.done();
        self
    }

    fn send_queue(&mut self, port: &mut PmdPort, queue: i32) -> Result<u32> {
        self.parent.send_queue(port, queue)
    }

    fn capacity(&self) -> i32 {
        self.parent.capacity()
    }
}

impl<T, V> BatchIterator for TransformBatch<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator + Act
{
    #[inline]
    fn start(&mut self) -> usize {
        self.parent.start()
    }

    #[inline]
    unsafe fn payload(&mut self, idx: usize) -> *mut u8 {
        self.parent.payload(idx)
    }

    #[inline]
    unsafe fn address(&mut self, idx: usize) -> *mut u8 {
        self.parent.address(idx)
    }

    #[inline]
    unsafe fn next_address(&mut self, idx: usize) -> Option<(*mut u8, usize)> {
        self.parent.next_address(idx)
    }

    #[inline]
    unsafe fn next_payload(&mut self, idx: usize) -> Option<(*mut u8, usize)> {
        self.parent.next_payload(idx)
    }

    #[inline]
    unsafe fn base_address(&mut self, idx: usize) -> *mut u8 {
        self.parent.base_address(idx)
    }

    #[inline]
    unsafe fn base_payload(&mut self, idx: usize) -> *mut u8 {
        self.parent.base_payload(idx)
    }

    #[inline]
    unsafe fn next_base_address(&mut self, idx: usize) -> Option<(*mut u8, usize)> {
        self.parent.next_base_address(idx)
    }

    #[inline]
    unsafe fn next_base_payload(&mut self, idx: usize) -> Option<(*mut u8, usize)> {
        self.parent.next_base_payload(idx)
    }
}
