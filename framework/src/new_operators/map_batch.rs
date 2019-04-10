use super::{Batch, PacketError};
use failure::Error;
use packets::Packet;

/// Lazily-evaluate map operator
///
/// On error, the packet is marked as aborted and will short-circuit the
/// remainder the pipeline.
pub struct MapBatch<B: Batch, T: Packet, M: FnMut(B::Item) -> Result<T, Error>> {
    source: B,
    map: M,
}

impl<B: Batch, T: Packet, M: FnMut(B::Item) -> Result<T, Error>> MapBatch<B, T, M> {
    pub fn new(source: B, map: M) -> Self {
        MapBatch { source, map }
    }
}

impl<B: Batch, T: Packet, M: FnMut(B::Item) -> Result<T, Error>> Batch for MapBatch<B, T, M> {
    type Item = T;

    fn next(&mut self) -> Option<Result<Self::Item, PacketError>> {
        self.source.next().map(|item| {
            match item {
                Ok(packet) => {
                    // TODO: can this be more efficient?
                    let mbuf = packet.mbuf();
                    (self.map)(packet).map_err(|e| PacketError::Abort(mbuf, e))
                }
                Err(e) => Err(e),
            }
        })
    }

    fn receive(&mut self) {
        self.source.receive();
    }
}
