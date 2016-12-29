use common::*;
use headers::EndOffset;
use interface::*;
use std::marker::PhantomData;
use super::Batch;
use super::act::Act;
use super::iterator::*;
use super::packet_batch::PacketBatch;

pub struct ParsedBatch<T, V>
    where T: EndOffset<PreviousHeader = V::Header>,
          V: Batch + BatchIterator + Act
{
    parent: V,
    phantom: PhantomData<T>,
}

impl<T, V> Act for ParsedBatch<T, V>
    where T: EndOffset<PreviousHeader = V::Header>,
          V: Batch + BatchIterator + Act
{
    act!{}
}

impl<T, V> Batch for ParsedBatch<T, V>
    where V: Batch + BatchIterator + Act,
          T: EndOffset<PreviousHeader = V::Header>
{
}

impl<T, V> ParsedBatch<T, V>
    where V: Batch + BatchIterator + Act,
          T: EndOffset<PreviousHeader = V::Header>
{
    #[inline]
    pub fn new(parent: V) -> ParsedBatch<T, V> {
        ParsedBatch {
            parent: parent,
            phantom: PhantomData,
        }
    }
}

impl<T, V> BatchIterator for ParsedBatch<T, V>
    where V: Batch + BatchIterator + Act,
          T: EndOffset<PreviousHeader = V::Header>
{
    type Header = T;
    type Metadata = V::Metadata;
    unsafe fn next_payload(&mut self, idx: usize) -> Option<PacketDescriptor<T, V::Metadata>> {
        self.parent.next_payload(idx).map(|p| PacketDescriptor { packet: p.packet.parse_header() })
    }

    #[inline]
    fn start(&mut self) -> usize {
        self.parent.start()
    }
}
