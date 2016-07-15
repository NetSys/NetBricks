use common::*;
use interface::*;
use headers::EndOffset;
use std::marker::PhantomData;
use super::act::Act;
use super::Batch;
use super::iterator::*;
use super::packet_batch::PacketBatch;

pub struct ParsedBatch<T, V>
    where T: EndOffset<PreviousHeader=V::Header>,
          V: Batch + BatchIterator + Act,
         
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

//batch!{ParsedBatch, [parent: V], [phantom: PhantomData]}
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
        ParsedBatch { parent: parent, phantom: PhantomData }
    }
}

impl<T, V> BatchIterator for ParsedBatch<T, V>
    where V: Batch + BatchIterator + Act,
          T: EndOffset<PreviousHeader = V::Header>
{
    type Header = T;
    unsafe fn next_payload(&mut self, idx: usize) -> Option<PacketDescriptor<T>> {
        let parent_payload = self.parent.next_payload(idx);
        match parent_payload {
            Some(PacketDescriptor { packet } ) =>
                packet.parse_header().and_then(|p| Some(PacketDescriptor {packet: p})),
            None => None
        }
    }
    #[inline]
    fn start(&mut self) -> usize {
        self.parent.start()
    }
}
