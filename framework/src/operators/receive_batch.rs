use super::act::Act;
use super::iterator::*;
use super::packet_batch::PacketBatch;
use super::Batch;
use common::*;
use headers::NullHeader;
use interface::{PacketRx, PacketTx};

pub struct ReceiveBatch<T: PacketRx> {
    parent: PacketBatch,
    queue: T,
    pub received: u64,
}

impl<T: PacketRx> ReceiveBatch<T> {
    pub fn new_with_parent(parent: PacketBatch, queue: T) -> ReceiveBatch<T> {
        ReceiveBatch {
            parent: parent,
            queue: queue,
            received: 0,
        }
    }

    pub fn new(queue: T) -> ReceiveBatch<T> {
        ReceiveBatch {
            parent: PacketBatch::new(32),
            queue: queue,
            received: 0,
        }
    }
}

impl<T: PacketRx> Batch for ReceiveBatch<T> {}

impl<T: PacketRx> BatchIterator for ReceiveBatch<T> {
    type Header = NullHeader;
    type Metadata = EmptyMetadata;
    #[inline]
    fn start(&mut self) -> usize {
        self.parent.start()
    }

    #[inline]
    unsafe fn next_payload(
        &mut self,
        idx: usize,
    ) -> Option<PacketDescriptor<NullHeader, EmptyMetadata>> {
        self.parent.next_payload(idx)
    }
}

/// Internal interface for packets.
impl<T: PacketRx> Act for ReceiveBatch<T> {
    #[inline]
    fn act(&mut self) {
        self.parent.act();
        self.parent
            .recv(&self.queue)
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
    fn send_q(&mut self, port: &PacketTx) -> Result<u32> {
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

    #[inline]
    fn get_task_dependencies(&self) -> Vec<usize> {
        self.parent.get_task_dependencies()
    }
}
