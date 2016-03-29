use super::act::Act;
use super::Batch;
use super::packet_batch::PacketBatch;
use super::iterator::*;
use super::super::pmd::*;
use super::super::interface::Result;
use std::any::Any;

// FIXME: Should we be handling multiple queues and ports here?
pub struct ReceiveBatch {
    parent: PacketBatch,
    port: PmdPort,
    queue: i32,
    pub received: u64,
}

impl ReceiveBatch {
    pub fn new_with_parent(parent: PacketBatch, port: PmdPort, queue: i32) -> ReceiveBatch {
        ReceiveBatch {
            parent: parent,
            port: port,
            queue: queue,
            received: 0,
        }
    }

    pub fn new(port: PmdPort, queue: i32) -> ReceiveBatch {
        ReceiveBatch {
            parent: PacketBatch::new(32),
            port: port,
            queue: queue,
            received: 0,
        }

    }
}

impl Batch for ReceiveBatch {}

impl BatchIterator for ReceiveBatch {
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

/// Internal interface for packets.
impl Act for ReceiveBatch {
    #[inline]
    fn act(&mut self) {
        self.parent.act();
        self.parent
            .recv_queue(&mut self.port, self.queue)
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
