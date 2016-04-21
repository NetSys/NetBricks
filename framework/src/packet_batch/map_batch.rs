use io::PortQueue;
use headers::EndOffset;
use io::Result;
use super::iterator::*;
use super::act::Act;
use super::Batch;
use super::HeaderOperations;
use std::any::Any;

pub type MapFn<T> = Box<FnMut(&T, &[u8], Option<&mut Any>)>;

pub struct MapBatch<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator + Act
{
    parent: V,
    transformer: MapFn<T>,
}

batch!{MapBatch, [parent: V, transformer: MapFn<T>], []}

impl<T, V> Act for MapBatch<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator + Act
{
    #[inline]
    fn parent(&mut self) -> &mut Batch{
        &mut self.parent
    }

    #[inline]
    fn parent_immutable(&self) -> &Batch {
        &self.parent
    }
    #[inline]
    fn act(&mut self) {
        self.parent.act();
        {
            let iter = PayloadEnumerator::<T>::new(&mut self.parent);
            while let Some(ParsedDescriptor { header: head, payload, ctx, .. }) = iter.next(&mut self.parent) {
                (self.transformer)(head, payload, ctx);
            }
        }
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

impl<T, V> BatchIterator for MapBatch<T, V>
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
