/// FIXME: This should replace receive queue eventually.
use common::*;
use utils::ReceivableQueue;
use interface::PortQueue;
use super::act::Act;
use super::Batch;
use super::packet_batch::PacketBatch;
use super::iterator::*;
use std::any::Any;
use headers::NullHeader;

pub struct ReceiveQueueGen<T: ReceivableQueue + Send> {
    parent: PacketBatch,
    queue: T,
    pub received: u64,
}

impl<T: ReceivableQueue + Send> ReceiveQueueGen<T> {
    pub fn new_with_parent(parent: PacketBatch, queue: T) -> ReceiveQueueGen<T> {
        ReceiveQueueGen {
            parent: parent,
            queue: queue,
            received: 0,
        }
    }

    pub fn new(queue: T) -> ReceiveQueueGen<T> {
        ReceiveQueueGen {
            parent: PacketBatch::new(32),
            queue: queue,
            received: 0,
        }

    }
}

impl<T: ReceivableQueue + Send> Batch for ReceiveQueueGen<T> {
    type Header = NullHeader;
}

impl<T: ReceivableQueue + Send> BatchIterator for ReceiveQueueGen<T> {
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

/// Internal interface for packets.
impl<T: ReceivableQueue + Send> Act for ReceiveQueueGen<T> {
    #[inline]
    fn parent(&mut self) -> &mut Act {
        &mut self.parent
    }

    #[inline]
    fn parent_immutable(&self) -> &Act {
        &self.parent
    }

    #[inline]
    fn act(&mut self) {
        self.parent.act();
        self.parent
            .recv_queue(&self.queue)
            .and_then(|x| {
                self.received += x as u64;
                Ok(x)
            })
            .expect("Receive failure");
    }

    #[inline]
    fn done(&mut self) {
        // Free up memory
        self.parent.deallocate_batch().expect("Deallocation failed");
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
