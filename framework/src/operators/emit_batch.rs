use super::{Batch, PacketError};
use packets::Packet;

/// Lazily-evaluated emit operator
///
/// Interrupts processing with a short-circuit error that simply emits the packet
pub struct EmitBatch<B: Batch> {
    source: B
}

impl<B: Batch> EmitBatch<B> {
    #[inline]
    pub fn new(source: B) -> Self {
        EmitBatch { source }
    }
}

impl<B: Batch> Batch for EmitBatch<B> {
    type Item = B::Item;

    #[inline]
    fn next(&mut self) -> Option<Result<Self::Item, PacketError>> {
        self.source.next().map(|item| match item {
            Ok(packet) => Err(PacketError::Emit(packet.mbuf())),
            e @ Err(_) => e,
        })
    }


    #[inline]
    fn receive(&mut self) {
        self.source.receive();
    }
}
