use super::{Batch, PacketError};
use packets::Packet;

/// Lazily-evaluated filter operator
///
/// If the predicate evaluates to `false`, the packet is marked as
/// dropped and will short-circuit the remainder of the pipeline.
pub struct FilterBatch<B: Batch, P>
where
    P: FnMut(&B::Item) -> bool,
{
    source: B,
    predicate: P,
}

impl<B: Batch, P> FilterBatch<B, P>
where
    P: FnMut(&B::Item) -> bool,
{
    #[inline]
    pub fn new(source: B, predicate: P) -> Self {
        FilterBatch { source, predicate }
    }
}

impl<B: Batch, P> Batch for FilterBatch<B, P>
where
    P: FnMut(&B::Item) -> bool,
{
    type Item = B::Item;

    #[inline]
    fn next(&mut self) -> Option<Result<Self::Item, PacketError>> {
        self.source.next().map(|item| match item {
            Ok(packet) => {
                if (self.predicate)(&packet) {
                    Ok(packet)
                } else {
                    Err(PacketError::Drop(packet.mbuf()))
                }
            }
            e @ Err(_) => e,
        })
    }

    #[inline]
    fn receive(&mut self) {
        self.source.receive();
    }
}
