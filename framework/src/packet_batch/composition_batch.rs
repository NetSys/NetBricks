use super::act::Act;
use super::Batch;
use super::iterator::{BatchIterator, PacketDescriptor};
use io::PortQueue;
use io::Result;
use std::any::Any;

/// `CompositionBatch` allows multiple NFs to be combined. A composition batch resets the packet pointer so that each NF
/// can treat packets as originating from the NF itself.
pub struct CompositionBatch {
    parent: Box<Batch>,
}

impl CompositionBatch {
    pub fn new(parent: Box<Batch>) -> CompositionBatch {
        CompositionBatch { parent: parent }

    }

    #[inline]
    pub fn process(&mut self) {
        self.act();
        self.done();
    }
}

impl Batch for CompositionBatch {}

impl BatchIterator for CompositionBatch {
    #[inline]
    fn start(&mut self) -> usize {
        self.parent.start()
    }

    #[inline]
    unsafe fn next_payload(&mut self, idx: usize) -> Option<(PacketDescriptor, Option<&mut Any>, usize)> {
        self.parent.next_base_payload(idx)
    }

    #[inline]
    unsafe fn next_base_payload(&mut self, idx: usize) -> Option<(PacketDescriptor, Option<&mut Any>, usize)> {
        self.parent.next_base_payload(idx)
    }

    #[inline]
    unsafe fn next_payload_popped(&mut self, _: usize, _: i32) -> Option<(PacketDescriptor, Option<&mut Any>, usize)> {
        panic!("Cannot pop beyond a composition batch")
    }
}

/// Internal interface for packets.
impl Act for CompositionBatch {
    act!{}
}
