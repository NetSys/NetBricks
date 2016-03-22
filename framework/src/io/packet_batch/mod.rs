pub use self::packet_batch::PacketBatch;
pub use self::parsed_batch::ParsedBatch;
pub use self::transform_batch::TransformBatch;
pub use self::apply_batch::ReplaceBatch;
pub use self::receive_batch::ReceiveBatch;
pub use self::send_batch::SendBatch;
pub use self::map_batch::MapBatch;
use super::interface::EndOffset;
use self::iterator::BatchIterator;
use super::pmd::*;
use self::act::Act;

#[macro_use]
mod macros;

mod packet_batch;
mod parsed_batch;
mod transform_batch;
mod receive_batch;
mod apply_batch;
mod send_batch;
mod iterator;
mod act;
mod map_batch;

/// Public interface implemented by every packet batch type.
pub trait Batch : Sized + BatchIterator + Act {
    type Header : EndOffset;
    type Parent : BatchIterator + Batch + Act;

    /// Parse the payload as header of type.
    fn parse<T: EndOffset>(self) -> ParsedBatch<T, Self> {
        ParsedBatch::<T, Self>::new(self)
    }

    /// Transform a header field.
    fn transform<'a>(self, transformer: &'a mut FnMut(&mut Self::Header)) -> TransformBatch<Self::Header, Self> {
        TransformBatch::<Self::Header, Self>::new(self, transformer)
    }

    /// Map over a set of header fields. Map and transform primarily differ in map being immutable. Immutability
    /// provides some optimization opportunities not otherwise available.
    fn map<'a>(self, transformer: &'a mut FnMut(&Self::Header)) -> MapBatch<Self::Header, Self> {
        MapBatch::<Self::Header, Self>::new(self, transformer)
    }

    /// Rewrite the entire header.
    fn replace(self, template: Self::Header) -> ReplaceBatch<Self::Header, Self> {
        ReplaceBatch::<Self::Header, Self>::new(self, template)
    }

    /// Get parent batch.
    fn pop(&mut self) -> &mut Self::Parent;

    /// Send this batch out a particular port and queue.
    fn send<'a>(self, port: &'a mut PmdPort, queue: i32) -> SendBatch<Self> {
        SendBatch::<Self>::new(self, port, queue)
    }
}
