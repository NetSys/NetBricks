use super::iterator::{BatchIterator, PacketBatchEnumerator};
use super::act::Act;
use super::Batch;
use super::HeaderOperations;
use super::super::interface::EndOffset;
use super::super::interface::Result;
use super::super::pmd::*;

pub struct FilterBatch<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator + Act
{
    parent: V,
    filter: Box<FnMut(&T) -> bool>,
    capacity: usize,
}

impl<T, V> FilterBatch<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator + Act
{
    #[inline]
    pub fn new(parent: V, filter: Box<FnMut(&T) -> bool>) -> FilterBatch<T, V> {
        let capacity = parent.capacity() as usize;
        FilterBatch {
            parent: parent,
            filter: filter,
            capacity: capacity,
        }
    }
}

batch_no_new!{FilterBatch}

impl<T, V> Act for FilterBatch<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator + Act
{
    #[inline]
    fn act(&mut self) -> &mut Self {
        self.parent.act();
        let mut remove = Vec::<usize>::with_capacity(self.capacity);
        {
            let ref mut f = self.filter;
            let iter = PacketBatchEnumerator::<T>::new(&mut self.parent);
            for (idx, packet) in iter {
                if !f(packet) {
                    remove.push(idx)
                }
            }
        }
        if remove.len() > 0 {
            self.parent.drop_packets(remove).expect("Filtering was performed incorrectly");
        }
        self
    }

    #[inline]
    fn done(&mut self) -> &mut Self {
        self.parent.done();
        self
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

impl<T, V> BatchIterator for FilterBatch<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator + Act
{
    #[inline]
    fn start(&mut self) -> usize {
        self.parent.start()
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
    unsafe fn next_base_address(&mut self, idx: usize) -> Option<(*mut u8, usize)> {
        self.parent.next_base_address(idx)
    }

    #[inline]
    unsafe fn next_base_payload(&mut self, idx: usize) -> Option<(*mut u8, usize)> {
        self.parent.next_base_payload(idx)
    }
}
