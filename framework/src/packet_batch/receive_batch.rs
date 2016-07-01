use io::PortQueue;
use io::Result;
use super::act::Act;
use super::Batch;
use super::packet_batch::PacketBatch;
use super::iterator::*;
use std::any::Any;
use headers::NullHeader;

// FIXME: Should we be handling multiple queues and ports here?
pub struct ReceiveBatch {
    parent: PacketBatch,
    port: PortQueue,
    pub received: u64,
}

impl ReceiveBatch {
    pub fn new_with_parent(parent: PacketBatch, port: PortQueue) -> ReceiveBatch {
        ReceiveBatch {
            parent: parent,
            port: port,
            received: 0,
        }
    }

    pub fn new(port: PortQueue) -> ReceiveBatch {
        ReceiveBatch {
            parent: PacketBatch::new(32),
            port: port,
            received: 0,
        }

    }
}

impl Batch for ReceiveBatch
{
    type Header = NullHeader;
}

impl BatchIterator for ReceiveBatch {
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
impl Act for ReceiveBatch {
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
            .recv(&mut self.port)
            .and_then(|x| {
                self.received += x as u64;
                Ok(x)
            })
            .expect("Receive failed");
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
