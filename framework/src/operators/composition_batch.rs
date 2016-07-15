use common::*;
use super::act::Act;
use super::Batch;
use super::iterator::{BatchIterator, PacketDescriptor};
use interface::PortQueue;
use scheduler::Executable;
use headers::EndOffset;
use headers::NullHeader;
use super::packet_batch::PacketBatch;

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
}

impl<T> BatchIterator for CompositionBatch<T>
    where T: EndOffset
{
    type Header = NullHeader;

    #[inline]
    fn start(&mut self) -> usize {
        self.parent.start()
    }

    #[inline]
    unsafe fn next_payload(&mut self, idx: usize) -> Option<PacketDescriptor<NullHeader>> {
        match self.parent.next_payload(idx) {
            Some(PacketDescriptor { packet }) => Some(PacketDescriptor{ packet: packet.reset() }),
            None => None
        }
    }

}

/// Internal interface for packets.
impl<T> Act for CompositionBatch<T>
    where T: EndOffset
{
    act!{}
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
