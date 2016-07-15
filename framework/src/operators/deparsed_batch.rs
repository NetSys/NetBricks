use common::*;
use std::marker::PhantomData;
use super::act::Act;
use super::Batch;
use super::iterator::{BatchIterator, PacketDescriptor};
use interface::PortQueue;
use headers::EndOffset;
use utils::SpscProducer;

pub struct DeparsedBatch<T: EndOffset, V>
    where V: Batch + BatchIterator + Act
{
    parent: V,
    phantom: PhantomData<T>,
}

impl<T, V> Act for DeparsedBatch<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator + Act
{
    act!{}
}

batch!{DeparsedBatch, [parent: V], [phantom: PhantomData]}

impl<T, V> BatchIterator for DeparsedBatch<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator + Act
{
    #[inline]
    fn start(&mut self) -> usize {
        self.parent.start()
    }

    #[inline]
    unsafe fn next_payload(&mut self, idx: usize) -> Option<(PacketDescriptor, Option<&mut Any>, usize)> {
        self.next_payload_popped(idx, 1)
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
        self.parent.next_payload_popped(idx, pop + 1)
    }
}
