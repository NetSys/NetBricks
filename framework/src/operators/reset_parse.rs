use common::*;
use interface::PortQueue;
use super::act::Act;
use super::Batch;
use super::iterator::*;
use headers::NullHeader;
use super::packet_batch::PacketBatch;

pub struct ResetParsingBatch<V>
    where V: Batch + BatchIterator + Act
{
    parent: V,
}

impl<V> ResetParsingBatch<V>
    where V: Batch + BatchIterator + Act
{
    pub fn new(parent: V) -> ResetParsingBatch<V> {
        ResetParsingBatch { parent: parent }
    }
}

impl<V> BatchIterator for ResetParsingBatch<V>
    where V: Batch + BatchIterator + Act
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
impl<V> Act for ResetParsingBatch<V>
    where V: Batch + BatchIterator + Act
{
    act!{}
}

impl<V> Batch for ResetParsingBatch<V> where V: Batch + BatchIterator + Act {}
