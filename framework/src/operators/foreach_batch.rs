use super::{Batch, PacketError};
use crate::packets::Packet;
use failure::Error;

/// Lazily-evaluate foreach operator
///
/// Works on reference of packet for side-effects.
///
/// On error, the packet is marked as aborted and will short-circuit the
/// remainder of the pipeline.
pub struct ForEachBatch<B: Batch, F>
where
    F: FnMut(&B::Item) -> Result<(), Error>,
{
    source: B,
    fun: F,
}

impl<B: Batch, F> ForEachBatch<B, F>
where
    F: FnMut(&B::Item) -> Result<(), Error>,
{
    #[inline]
    pub fn new(source: B, fun: F) -> Self {
        ForEachBatch { source, fun }
    }
}

impl<B: Batch, F> Batch for ForEachBatch<B, F>
where
    F: FnMut(&B::Item) -> Result<(), Error>,
{
    type Item = B::Item;

    #[inline]
    fn next(&mut self) -> Option<Result<Self::Item, PacketError>> {
        self.source.next().map(|item| match item {
            Ok(packet) => match (self.fun)(&packet) {
                Ok(_) => Ok(packet),
                Err(e) => Err(PacketError::Abort(packet.mbuf(), e)),
            },
            Err(e) => Err(e),
        })
    }

    #[inline]
    fn receive(&mut self) {
        self.source.receive();
    }
}
