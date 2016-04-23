use super::act::Act;
use super::Batch;
use super::Executable;
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
    #[inline]
    fn parent(&mut self) -> &mut Batch {
        &mut *self.parent
    }

    #[inline]
    fn parent_immutable(&self) -> &Batch {
        &*self.parent
    }

    #[inline]
    fn act(&mut self) {
        self.parent.act();
    }

    #[inline]
    fn done(&mut self) {
        self.parent.done();
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

impl Executable for CompositionBatch {
    #[inline]
    fn execute(&mut self) {
        self.act();
        self.done();
    }
}
