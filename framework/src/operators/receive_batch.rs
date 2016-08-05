use common::*;
use interface::PortQueue;
use super::act::Act;
use super::Batch;
use super::packet_batch::PacketBatch;
use super::iterator::*;
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

impl Batch for ReceiveBatch {}

impl BatchIterator for ReceiveBatch {
    type Header = NullHeader;
    #[inline]
    fn start(&mut self) -> usize {
        self.parent.start()
    }

    #[inline]
    unsafe fn next_payload(&mut self, idx: usize) -> Option<PacketDescriptor<NullHeader>> {
        self.parent.next_payload(idx)
        //self.parent.next_payload(idx).map(|mut p| { p.packet.reset_stack_offset(); p })
    }
}

/// Internal interface for packets.
impl Act for ReceiveBatch {
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
    fn drop_packets(&mut self, idxes: &[usize]) -> Option<usize> {
        self.parent.drop_packets(idxes)
    }

    #[inline]
    fn clear_packets(&mut self) {
        self.parent.clear_packets()
    }

    #[inline]
    fn get_packet_batch(&mut self) -> &mut PacketBatch {
        &mut self.parent
    }
}
