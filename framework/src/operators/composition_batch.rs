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
pub struct CompositionBatch<T: EndOffset, M: Sized + Send> {
    parent: Box<Batch<Header = T, Metadata = M>>,
}

impl<T, M> CompositionBatch<T, M>
    where T: EndOffset,
          M: Sized + Send
{
    pub fn new(parent: Box<Batch<Header = T, Metadata = M>>) -> CompositionBatch<T, M> {
        CompositionBatch { parent: parent }

    }
}

impl<T, M> Batch for CompositionBatch<T, M>
    where T: EndOffset,
          M: Sized + Send
{
}

impl<T, M> BatchIterator for CompositionBatch<T, M>
    where T: EndOffset,
          M: Sized + Send
{
    type Header = NullHeader;
    type Metadata = EmptyMetadata;

    #[inline]
    fn start(&mut self) -> usize {
        self.parent.start()
    }

    #[inline]
    unsafe fn next_payload(&mut self, idx: usize) -> Option<PacketDescriptor<NullHeader, EmptyMetadata>> {
        match self.parent.next_payload(idx) {
            Some(PacketDescriptor { packet }) => Some(PacketDescriptor { packet: packet.reset() }),
            None => None,
        }
    }
}

/// Internal interface for packets.
impl<T, M> Act for CompositionBatch<T, M>
    where T: EndOffset,
          M: Sized + Send
{
    act!{}
}

impl<T, M> Executable for CompositionBatch<T, M>
    where T: EndOffset,
          M: Sized + Send
{
    #[inline]
    fn execute(&mut self) {
        self.act();
        self.done();
    }
}
