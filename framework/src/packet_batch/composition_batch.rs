use super::act::Act;
use super::Batch;
use super::iterator::{BatchIterator, PacketDescriptor};
use io::PortQueue;
use io::Result;
use std::any::Any;
use scheduler::Executable;
use headers::EndOffset;
use headers::NullHeader;

/// `CompositionBatch` allows multiple NFs to be combined. A composition batch resets the packet pointer so that each NF
/// can treat packets as originating from the NF itself.
pub struct CompositionBatch<T: EndOffset> {
    parent: Box<Batch<Header = T>>,
}

impl<T> CompositionBatch<T>
    where T: EndOffset
{
    pub fn new(parent: Box<Batch<Header = T>>) -> CompositionBatch<T> {
        CompositionBatch { parent: parent }

    }
}

impl<T> Batch for CompositionBatch<T>
    where T: EndOffset
{
    type Header = NullHeader;
}

impl<T> BatchIterator for CompositionBatch<T>
    where T: EndOffset
{
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
impl<T> Act for CompositionBatch<T>
    where T: EndOffset
{
    #[inline]
    fn parent(&mut self) -> &mut Act {
        panic!("Cannot use parent to work through CompositionBatch")
    }

    #[inline]
    fn parent_immutable(&self) -> &Act {
        panic!("Cannot use parent to work through CompositionBatch")
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

impl<T> Executable for CompositionBatch<T>
    where T: EndOffset
{
    #[inline]
    fn execute(&mut self) {
        self.act();
        self.done();
    }
}
