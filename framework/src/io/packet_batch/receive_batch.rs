use super::act::Act;
use super::Batch;
use super::PacketBatch;
use super::TransformBatch;
use super::ReplaceBatch;
use super::iterator::BatchIterator;
use super::super::pmd::*;
use super::super::super::headers::NullHeader;
use super::super::interface::Result;

// FIXME: Should we be handling multiple queues and ports here?
pub struct ReceiveBatch {
    parent: PacketBatch,
    port: PmdPort,
    queue: i32,
    pub received: u64,
}

impl ReceiveBatch {
    pub fn new(parent: PacketBatch, port: PmdPort, queue: i32) -> ReceiveBatch {
        ReceiveBatch {
            parent: parent,
            port: port,
            queue: queue,
            received: 0,
        }
    }
}

impl Batch for ReceiveBatch {
    type Header = NullHeader;
    type Parent = PacketBatch;

    fn pop(&mut self) -> &mut PacketBatch {
        &mut self.parent
    }

    fn transform(self, _: &mut FnMut(&mut NullHeader)) -> TransformBatch<NullHeader, Self> {
        panic!("Cannot transform ReceiveBatch")
    }

    fn replace(self, _: NullHeader) -> ReplaceBatch<NullHeader, Self> {
        panic!("Cannot replace ReceiveBatch")
    }
}

impl BatchIterator for ReceiveBatch {
    #[inline]
    fn start(&mut self) -> usize {
        self.parent.start()
    }

    #[inline]
    unsafe fn payload(&mut self, idx: usize) -> *mut u8 {
        self.parent.payload(idx)
    }

    #[inline]
    unsafe fn address(&mut self, idx: usize) -> *mut u8 {
        self.parent.address(idx)
    }

    #[inline]
    unsafe fn next_address(&mut self, idx: usize) -> Option<(*mut u8, usize)> {
        self.parent.next_address(idx)
    }

    #[inline]
    unsafe fn next_payload(&mut self, idx: usize) -> Option<(*mut u8, usize)> {
        self.parent.next_payload(idx)
    }

    #[inline]
    unsafe fn base_address(&mut self, idx: usize) -> *mut u8 {
        self.parent.base_address(idx)
    }

    #[inline]
    unsafe fn base_payload(&mut self, idx: usize) -> *mut u8 {
        self.parent.base_payload(idx)
    }

    #[inline]
    unsafe fn next_base_address(&mut self, idx: usize) -> Option<(*mut u8, usize)> {
        self.parent.next_base_address(idx)
    }

    #[inline]
    unsafe fn next_base_payload(&mut self, idx: usize) -> Option<(*mut u8, usize)> {
        self.parent.next_base_payload(idx)
    }
}

/// Internal interface for packets.
impl Act for ReceiveBatch {
    #[inline]
    fn act(&mut self) -> &mut Self {
        self.parent.act();
        self.parent
            .recv_queue(&mut self.port, self.queue)
            .and_then(|x| {
                self.received += x as u64;
                Ok(x)
            })
            .expect("Receive failed");
        self
    }

    fn done(&mut self) -> &mut Self {
        // Free up memory
        self.parent.deallocate_batch().expect("Deallocation failed");
        self
    }

    fn send_queue(&mut self, port: &mut PmdPort, queue: i32) -> Result<u32> {
        self.parent.send_queue(port, queue)
    }

    fn capacity(&self) -> i32 {
        self.parent.capacity()
    }
}
