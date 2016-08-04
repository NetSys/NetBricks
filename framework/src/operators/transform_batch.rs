use common::*;
use interface::PortQueue;
use interface::Packet;
use headers::EndOffset;
use super::iterator::*;
use super::act::Act;
use super::Batch;
use std::marker::PhantomData;
use super::packet_batch::PacketBatch;

pub type TransformFn<T> = Box<FnMut(&mut Packet<T>) + Send>;

pub struct TransformBatch<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator<Header = T> + Act
{
    parent: V,
    transformer: TransformFn<T>,
    applied: bool,
    phantom_t: PhantomData<T>,
}

impl<T, V> TransformBatch<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator<Header = T> + Act
{
    pub fn new(parent: V, transformer: TransformFn<T>) -> TransformBatch<T, V> {
        TransformBatch {
            parent: parent,
            transformer: transformer,
            applied: false,
            phantom_t: PhantomData,
        }
    }
}

impl<T, V> Batch for TransformBatch<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator<Header = T> + Act
{
}

impl<T, V> BatchIterator for TransformBatch<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator<Header = T> + Act
{
    type Header = T;
    #[inline]
    fn start(&mut self) -> usize {
        self.parent.start()
    }

    #[inline]
    unsafe fn next_payload(&mut self, idx: usize) -> Option<PacketDescriptor<T>> {
        self.parent.next_payload(idx)
    }
}

impl<T, V> Act for TransformBatch<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator<Header = T> + Act
{
    #[inline]
    fn act(&mut self) {
        if !self.applied {
            self.parent.act();
            {
                let iter = PayloadEnumerator::<T>::new(&mut self.parent);
                while let Some(ParsedDescriptor { mut packet, .. }) = iter.next(&mut self.parent) {
                    (self.transformer)(&mut packet);
                }
            }
            self.applied = true;
        }
    }

    #[inline]
    fn done(&mut self) {
        self.applied = false;
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
    fn drop_packets(&mut self, idxes: &[usize]) -> Option<usize> {
        self.parent.drop_packets(idxes)
    }

    #[inline]
    fn clear_packets(&mut self) {
        self.parent.clear_packets()
    }

    #[inline]
    fn get_packet_batch(&mut self) -> &mut PacketBatch {
        self.parent.get_packet_batch()
    }
}
