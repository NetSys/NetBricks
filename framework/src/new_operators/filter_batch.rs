use super::{Batch, PacketError};
use packets::Packet;

/// Lazily-evaluated filter operator
///
/// If the predicate evaluates to `false`, the packet is marked as
/// dropped and will short-circuit the remainder the pipeline.
pub struct FilterBatch<B: Batch, P: FnMut(&B::Item) -> bool> {
    source: B,
    predicate: P,
}

impl<B: Batch, P: FnMut(&B::Item) -> bool> FilterBatch<B, P> {
    pub fn new(source: B, predicate: P) -> Self {
        FilterBatch { source, predicate }
    }
}

impl<B: Batch, P: FnMut(&B::Item) -> bool> Batch for FilterBatch<B, P> {
    type Item = B::Item;

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

    fn receive(&mut self) {
        self.source.receive();
    }
}
