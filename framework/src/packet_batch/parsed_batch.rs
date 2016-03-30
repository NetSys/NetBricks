use io::PmdPort;
use headers::EndOffset;
use io::Result;
use std::marker::PhantomData;
use super::act::Act;
use super::Batch;
use super::HeaderOperations;
use super::iterator::*;
use super::packet_batch::cast_from_u8;
use std::any::Any;
use std::cmp::min;

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

    #[inline]
    fn adjust_payload_size(&mut self, idx: usize, size: isize) -> Option<isize> {
        self.parent.adjust_payload_size(idx, size)
    }

    #[inline]
    fn adjust_headroom(&mut self, idx: usize, size: isize) -> Option<isize> {
        self.parent.adjust_headroom(idx, size)
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
    unsafe fn next_payload(&mut self, idx: usize) -> Option<(PacketDescriptor, Option<&mut Any>, usize)> {
        let parent_payload = self.parent.next_payload(idx);
        match parent_payload {
            Some((PacketDescriptor { offset: prev_offset, payload: packet, payload_size: size, .. },
                  arg,
                  idx)) => {
                let pkt_as_t = cast_from_u8::<T>(packet);
                let offset = T::offset(pkt_as_t);
                // Under no circumstances should we allow an incorrectly reported payload size to cause problems.
                let payload_size = min(T::payload_size(pkt_as_t, size), size - offset);
                Some((PacketDescriptor {
                    header: packet,
                    offset: prev_offset + offset,
                    payload: packet.offset(offset as isize),
                    payload_size: payload_size,
                },
                      arg,
                      idx))
            }
            None => None,
        }
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
        // mark as likely (can do this with llvm expect intrinsic)
        if pop - 1 == 0 {
            self.next_payload(idx)
        } else {
            self.parent.next_payload_popped(idx, pop - 1)
        }
    }
}
