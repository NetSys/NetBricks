use super::Act;
use super::Batch;
use super::PacketBatch;
use super::TransformBatch;
use super::ReplaceBatch;
use super::iterator::BatchIterator;
use super::super::pmd::*;
use super::super::super::headers::NullHeader;
use super::super::interface::Result;

// FIXME: Should we be handling multiple queues and ports here?
pub struct ReceiveBatch<'a> {
    parent: &'a mut PacketBatch,
    applied: bool,
    port: &'a mut PmdPort,
    queue: i32,
    pub received: u64
}

impl<'a> ReceiveBatch<'a> {
    pub fn new(parent: &'a mut PacketBatch, port: &'a mut PmdPort, queue: i32) -> ReceiveBatch<'a> {
        ReceiveBatch{applied: false, parent: parent, port: port, queue: queue, received: 0}
    }
}

impl<'a> Batch for ReceiveBatch<'a> {
    type Header = NullHeader;
    type Parent = PacketBatch;
    
    fn pop(&mut self) -> &mut PacketBatch {
        self.parent
    }

    fn transform(&mut self, _: &mut FnMut(&mut NullHeader)) -> TransformBatch<NullHeader, Self> {
        panic!("Cannot transform ReceiveBatch")
    }

    fn replace(&mut self, _: &NullHeader) -> ReplaceBatch<NullHeader, Self> {
        panic!("Cannot replace ReceiveBatch")
    }
}

impl<'a> BatchIterator for ReceiveBatch<'a> {
    #[inline]
    fn start(&mut self) -> usize {
        self.act();
        self.parent.start()
    }

    #[inline]
    unsafe fn payload(&mut self, idx: usize) -> *mut u8 {
        self.act();
        self.parent.payload(idx)
    }

    #[inline]
    unsafe fn address(&mut self, idx: usize) -> *mut u8 {
        self.act();
        self.parent.address(idx)
    }

    #[inline]
    unsafe fn next_address(&mut self, idx: usize) -> Option<(*mut u8, usize)> {
        self.act();
        self.parent.next_address(idx)
    }

    #[inline]
    unsafe fn next_payload(&mut self, idx: usize) -> Option<(*mut u8, usize)> {
        self.act();
        self.parent.next_payload(idx)
    }
}

/// Internal interface for packets.
impl<'a> Act for ReceiveBatch<'a> {
    #[inline]
    fn act(&mut self) -> &mut Self {
        if !self.applied {
            self.parent.recv_queue(self.port, self.queue)
                .and_then(|x| {self.received += x as u64; Ok(x)}).expect("Receive failed");
            self.applied = true
        }
        self
    }

    fn done(&mut self) -> &mut Self {
        // Free up memory
        self.parent.deallocate_batch().expect("Deallocation failed");
        self.applied = false;
        self
    }

    fn send_queue(&mut self, port: &mut PmdPort, queue: i32) -> Result<u32> {
        self.parent.send_queue(port, queue)
    }
}
