use super::act::Act;
use super::iterator::*;
use super::packet_batch::PacketBatch;
use super::Batch;
use common::*;
use headers::EndOffset;
use interface::*;

pub struct DeparsedBatch<V>
where
    V: Batch + BatchIterator + Act,
{
    parent: V,
}

impl<V> Act for DeparsedBatch<V>
where
    V: Batch + BatchIterator + Act,
{
    act!{}
}

impl<V> Batch for DeparsedBatch<V> where V: Batch + BatchIterator + Act {}

impl<V> DeparsedBatch<V>
where
    V: Batch + BatchIterator + Act,
{
    #[inline]
    pub fn new(parent: V) -> DeparsedBatch<V> {
        DeparsedBatch { parent: parent }
    }
}

impl<V> BatchIterator for DeparsedBatch<V>
where
    V: Batch + BatchIterator + Act,
{
    type Header = <<V as BatchIterator>::Header as EndOffset>::PreviousHeader;
    type Metadata = <V as BatchIterator>::Metadata;
    unsafe fn next_payload(
        &mut self,
        idx: usize,
    ) -> Option<PacketDescriptor<Self::Header, Self::Metadata>> {
        self.parent.next_payload(idx).map(|p| PacketDescriptor {
            packet: p.packet.deparse_header_stack().unwrap(),
        })
    }

    #[inline]
    fn start(&mut self) -> usize {
        self.parent.start()
    }
}
