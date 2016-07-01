use io::PortQueue;
use io::Result;
use super::act::Act;
use super::Batch;
use super::iterator::*;
use std::any::Any;
use utils::SpscProducer;
use headers::NullHeader;

// FIXME: Reconsider this choice some day
/// This is really the same thing as composition except that by accepting a template it is somewhat faster (since we
/// don't need dynamic dispatch).
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

impl<V> Batch for ResetParsingBatch<V>
    where V: Batch + BatchIterator + Act
{
    type Header = NullHeader;
}

impl<V> BatchIterator for ResetParsingBatch<V>
    where V: Batch + BatchIterator + Act
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
        panic!("Cannot pop past a rest operation")
    }
}

/// Internal interface for packets.
impl<V> Act for ResetParsingBatch<V>
    where V: Batch + BatchIterator + Act
{
    act!{}
}
