use std::marker::PhantomData;
use super::act::Act;
use super::Batch;
use super::HeaderOperations;
use super::iterator::*;
use super::packet_batch::cast_from_u8;
use super::super::interface::EndOffset;
use super::super::pmd::*;
use super::super::interface::Result;
use std::any::Any;

pub struct ParsedBatch<T: EndOffset, V>
    where V: Batch + BatchIterator + Act
{
    parent: V,
    phantom: PhantomData<T>,
}

impl<T, V> Act for ParsedBatch<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator + Act
{
    #[inline]
    fn act(&mut self) {
        self.parent.act();
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

batch!{ParsedBatch, [parent: V], [phantom: PhantomData]}

impl<T, V> BatchIterator for ParsedBatch<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator + Act
{
    #[inline]
    fn start(&mut self) -> usize {
        self.parent.start()
    }

    #[inline]
    unsafe fn next_address(&mut self, idx: usize, pop: i32) -> address_iterator_return!{} {
        if pop > 1 {
            self.parent.next_address(idx, pop - 1)
        } else {
            match self.parent.next_payload(idx) {
                None => None,
                Some((_, payload, size, ctx, next)) => Some((payload, size, ctx, next)),
            }
        }
    }

    #[inline]
    unsafe fn next_payload(&mut self, idx: usize) -> payload_iterator_return!{} {
        let parent_payload = self.parent.next_payload(idx);
        match parent_payload {
            Some((_, packet, size, arg, idx)) => {
                let pkt_as_t = cast_from_u8::<T>(packet);
                let offset = T::offset(pkt_as_t);
                let payload_size = T::payload_size(pkt_as_t, size);
                Some((packet,
                      packet.offset(offset as isize),
                      payload_size,
                      arg,
                      idx))
            }
            None => None,
        }
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
        if pop - 1 == 0 {
            self.next_payload(idx)
        } else {
            self.parent.next_payload_popped(idx, pop - 1)
        }
    }
}
