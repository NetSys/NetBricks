use super::iterator::{BatchIterator, PacketBatchIterator};
use super::act::Act;
use super::Batch;
use super::HeaderOperations;
use super::super::pmd::*;
use super::super::interface::EndOffset;
use std::ptr;
use super::super::interface::Result;
use std::any::Any;

pub struct ReplaceBatch<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator + Act
{
    parent: V,
    template: T,
}

batch!{ReplaceBatch, [parent: V, template: T]}

impl<T, V> Act for ReplaceBatch<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator + Act
{
    #[inline]
    fn act(&mut self) {
        self.parent.act();
        let iter = PacketBatchIterator::<T>::new(&mut self.parent);
        while let Some((packet, _)) = iter.next(&mut self.parent) {
            unsafe {
                ptr::copy_nonoverlapping(&self.template, packet, 1);
            }
        }
    }

    #[inline]
    fn done(&mut self) {
        self.parent.done();
    }

    #[inline]
    fn send_queue(&mut self, port: &mut PmdPort, queue: i32) -> Result<u32> {
        self.parent.send_queue(port, queue)
    }

    #[inline]
    fn capacity(&self) -> i32 {
        self.parent.capacity()
    }

    #[inline]
    fn drop_packets(&mut self, idxes: Vec<usize>) -> Option<usize> {
        self.parent.drop_packets(idxes)
    }
}

impl<T, V> BatchIterator for ReplaceBatch<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator + Act
{
    #[inline]
    fn start(&mut self) -> usize {
        self.parent.start()
    }

    #[inline]
    unsafe fn next_address(&mut self, idx: usize, pop: i32) -> Option<(*mut u8, usize, Option<&mut Any>, usize)> {
        self.parent.next_address(idx, pop)
    }

    #[inline]
    unsafe fn next_payload(&mut self, idx: usize) -> Option<(*mut u8, *mut u8, usize, Option<&mut Any>, usize)> {
        self.parent.next_payload(idx)
    }

    #[inline]
    unsafe fn next_base_address(&mut self, idx: usize) -> Option<(*mut u8, usize, Option<&mut Any>, usize)> {
        self.parent.next_base_address(idx)
    }

    #[inline]
    unsafe fn next_base_payload(&mut self, idx: usize) -> Option<(*mut u8, *mut u8, usize, Option<&mut Any>, usize)> {
        self.parent.next_base_payload(idx)
    }

    #[inline]
    unsafe fn next_payload_popped(&mut self,
                                  idx: usize,
                                  pop: i32)
                                  -> Option<(*mut u8, *mut u8, usize, Option<&mut Any>, usize)> {
        self.parent.next_payload_popped(idx, pop)
    }
}
