use super::iterator::*;
use super::act::Act;
use super::Batch;
use super::HeaderOperations;
use io::PmdPort;
use headers::EndOffset;
use io::Result;
use std::any::Any;

/// Takes in the header, payload and context, and returns the difference between the current packet size and desired
/// packet size.
pub type ResizeFn<T> = Box<FnMut(&mut T, &[u8], Option<&mut Any>) -> isize>;

pub struct ResizePayload<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator + Act
{
    parent: V,
    resize_fn: ResizeFn<T>,
    capacity: usize,
}

impl<T, V> ResizePayload<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator + Act
{
    #[inline]
    pub fn new(parent: V, resize_fn: ResizeFn<T>) -> ResizePayload<T, V> {
        let capacity = parent.capacity() as usize;
        ResizePayload {
            parent: parent,
            resize_fn: resize_fn,
            capacity: capacity,
        }
    }
}

batch_no_new!{ResizePayload}

impl<T, V> Act for ResizePayload<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator + Act
{
    #[inline]
    fn act(&mut self) {
        self.parent.act();
        let mut idxes_sizes = Vec::<(usize, isize)>::with_capacity(self.capacity);
        {
            let iter = PayloadEnumerator::<T>::new(&mut self.parent);
            while let Some(ParsedDescriptor { index: idx, header: head, payload, ctx, .. }) =
                      iter.next(&mut self.parent) {
                let new_size = (self.resize_fn)(head, payload, ctx);
                if new_size != 0 {
                    idxes_sizes.push((idx, new_size))
                }
            }
        }
        for (idx, size) in idxes_sizes {
            // FIXME: Error handling, this currently just panics, but it should do something different. Maybe
            // take a failure handler or drop the packet instead of panicing?
            match self.parent.adjust_payload_size(idx, size) {
                Some(_) => (),
                None => panic!("Resize failed {}", idx),
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

    #[inline]
    fn adjust_payload_size(&mut self, idx: usize, size: isize) -> Option<isize> {
        self.parent.adjust_payload_size(idx, size)
    }

    #[inline]
    fn adjust_headroom(&mut self, idx: usize, size: isize) -> Option<isize> {
        self.parent.adjust_headroom(idx, size)
    }
}

impl<T, V> BatchIterator for ResizePayload<T, V>
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
