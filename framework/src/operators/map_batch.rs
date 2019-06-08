use super::{Batch, PacketError};
use crate::packets::Packet;
use failure::Error;

/// Lazily-evaluate map operator
///
/// On error, the packet is marked as aborted and will short-circuit the
/// remainder of the pipeline.
pub struct MapBatch<B: Batch, T: Packet, M>
where
    M: FnMut(B::Item) -> Result<T, Error>,
{
    source: B,
    map: M,
}

impl<B: Batch, T: Packet, M> MapBatch<B, T, M>
where
    M: FnMut(B::Item) -> Result<T, Error>,
{
    #[inline]
    pub fn new(source: B, map: M) -> Self {
        MapBatch { source, map }
    }
}

impl<B: Batch, T: Packet, M> Batch for MapBatch<B, T, M>
where
    M: FnMut(B::Item) -> Result<T, Error>,
{
    type Item = T;

    #[inline]
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

    #[inline]
    fn receive(&mut self) {
        self.source.receive();
    }
}
