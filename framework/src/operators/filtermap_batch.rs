use super::{Batch, PacketError};
use failure::Error;
use packets::Packet;

/// Lazily-evaluate filter_map operator
///
/// On error, the packet is marked as aborted and will short-circuit the
/// remainder of the pipeline.
pub struct FilterMapBatch<B: Batch, T: Packet, F>
where
    F: FnMut(B::Item) -> Result<Option<T>, Error>,
{
    source: B,
    f: F,
}

impl<B: Batch, T: Packet, F> FilterMapBatch<B, T, F>
where
    F: FnMut(B::Item) -> Result<Option<T>, Error>,
{
    #[inline]
    pub fn new(source: B, f: F) -> Self {
        FilterMapBatch { source, f }
    }
}

impl<B: Batch, T: Packet, F> Batch for FilterMapBatch<B, T, F>
where
    F: FnMut(B::Item) -> Result<Option<T>, Error>,
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Result<Self::Item, PacketError>> {
        self.source.next().map(|item| match item {
            Ok(packet) => {
                let mbuf = packet.mbuf();
                match (self.f)(packet) {
                    Ok(Some(p)) => Ok(p),
                    Ok(None) => Err(PacketError::Drop(mbuf)),
                    Err(e) => Err(PacketError::Abort(mbuf, e)),
                }
            }
            Err(e) => Err(e),
        })
    }

    #[inline]
    fn receive(&mut self) {
        self.source.receive();
    }
}
