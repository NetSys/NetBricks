use super::iterator::{BatchIterator, PayloadEnumerator};
use super::act::Act;
use super::Batch;
use super::HeaderOperations;
use super::super::interface::EndOffset;
use super::super::interface::Result;
use super::super::pmd::*;
use std::any::Any;

pub type FilterFn<T> = Box<FnMut(&T, &[u8], Option<&mut Any>) -> bool>;

pub struct FilterBatch<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator + Act
{
    parent: V,
    filter: FilterFn<T>,
    capacity: usize,
}

impl<T, V> FilterBatch<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator + Act
{
    #[inline]
    pub fn new(parent: V, filter: FilterFn<T>) -> FilterBatch<T, V> {
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
    fn act(&mut self) {
        self.parent.act();
        let mut remove = Vec::<usize>::with_capacity(self.capacity);
        {
            // let ref mut f = self.filter;
            let iter = PayloadEnumerator::<T>::new(&mut self.parent);
            while let Some((idx, head, payload, ctx)) = iter.next(&mut self.parent) {
                if (self.filter)(head, payload, ctx) {
                    remove.push(idx)
                }
            }
        }
        if !remove.is_empty() {
            self.parent.drop_packets(remove).expect("Filtering was performed incorrectly");
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

impl<T, V> BatchIterator for FilterBatch<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator + Act
{
    #[inline]
    fn start(&mut self) -> usize {
        self.parent.start()
    }

    #[inline]
    unsafe fn next_address(&mut self, idx: usize, pop: i32) -> address_iterator_return!{} {
        self.parent.next_address(idx, pop)
    }

    #[inline]
    unsafe fn next_payload(&mut self, idx: usize) -> payload_iterator_return!{} {
        self.parent.next_payload(idx)
    }

    #[inline]
    unsafe fn next_base_address(&mut self, idx: usize) -> address_iterator_return!{} {
        self.parent.next_base_address(idx)
    }

    #[inline]
    unsafe fn next_base_payload(&mut self, idx: usize) -> payload_iterator_return!{} {
        self.parent.next_base_payload(idx)
    }

    #[inline]
    unsafe fn next_payload_popped(&mut self, idx: usize, pop: i32) -> payload_iterator_return!{} {
        self.parent.next_payload_popped(idx, pop)
    }
}
