use super::iterator::*;
use super::act::Act;
use super::Batch;
use super::HeaderOperations;
use io::PortQueue;
use headers::EndOffset;
use io::Result;
use std::any::Any;

pub type FilterFn<T> = Box<FnMut(&T, &[u8], Option<&mut Any>) -> bool + Send>;

pub struct FilterBatch<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator + Act
{
    parent: V,
    filter: FilterFn<T>,
    capacity: usize,
    remove: Vec<usize>,
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
            remove: Vec::with_capacity(capacity),
        }
    }
}

batch_no_new!{FilterBatch}

impl<T, V> Act for FilterBatch<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator + Act
{
    #[inline]
    fn parent(&mut self) -> &mut Batch {
        &mut self.parent
    }

    #[inline]
    fn parent_immutable(&self) -> &Batch {
        &self.parent
    }
    #[inline]
    fn act(&mut self) {
        self.parent.act();
        // let ref mut f = self.filter;
        let iter = PayloadEnumerator::<T>::new(&mut self.parent);
        while let Some(ParsedDescriptor { index: idx, header: head, payload, ctx, .. }) = iter.next(&mut self.parent) {
            if (self.filter)(head, payload, ctx) {
                self.remove.push(idx)
            }
        }
        if !self.remove.is_empty() {
            self.parent.drop_packets(&self.remove).expect("Filtering was performed incorrectly");
        }
        self.remove.clear();
    }

    #[inline]
    fn done(&mut self) {
        self.parent.done();
    }

    #[inline]
    fn send_q(&mut self, port: &mut PortQueue) -> Result<u32> {
        self.parent.send_q(port)
    }

    #[inline]
    fn capacity(&self) -> i32 {
        self.capacity as i32
    }

    #[inline]
    fn drop_packets(&mut self, idxes: &Vec<usize>) -> Option<usize> {
        self.parent.drop_packets(idxes)
    }

    #[inline]
    fn adjust_payload_size(&mut self, idx: usize, size: isize) -> Option<isize> {
        self.parent.adjust_payload_size(idx, size)
    }

    #[inline]
    fn adjust_headroom(&mut self, idx: usize, size: isize) -> Option<isize> {
        self.parent.adjust_headroom(idx, size)
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
    unsafe fn next_payload(&mut self, idx: usize) -> Option<(PacketDescriptor, Option<&mut Any>, usize)> {
        self.parent.next_payload(idx)
    }

    #[inline]
    unsafe fn next_base_payload(&mut self, idx: usize) -> Option<(PacketDescriptor, Option<&mut Any>, usize)> {
        self.parent.next_base_payload(idx)
    }

    #[inline]
    unsafe fn next_payload_popped(&mut self,
                                  idx: usize,
                                  pop: i32)
                                  -> Option<(PacketDescriptor, Option<&mut Any>, usize)> {
        self.parent.next_payload_popped(idx, pop)
    }
}
