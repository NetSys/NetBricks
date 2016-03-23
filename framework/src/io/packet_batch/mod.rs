use self::act::Act;
use self::iterator::BatchIterator;
pub use self::parsed_batch::ParsedBatch;
pub use self::transform_batch::TransformBatch;
pub use self::apply_batch::ReplaceBatch;
pub use self::receive_batch::ReceiveBatch;
pub use self::send_batch::SendBatch;
pub use self::map_batch::MapBatch;
pub use self::composition_batch::CompositionBatch;
use super::interface::EndOffset;
use super::pmd::*;

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
mod composition_batch;

/// Public interface implemented by every packet batch type.
pub trait Batch : Sized + BatchIterator + Act {
    type Parent : BatchIterator + Batch + Act;

    /// Parse the payload as header of type.
    fn parse<T: EndOffset>(self) -> ParsedBatch<T, Self> {
        ParsedBatch::<T, Self>::new(self)
    }

    /// Send this batch out a particular port and queue.
    fn send<'a>(self, port: &'a mut PmdPort, queue: i32) -> SendBatch<Self> {
        SendBatch::<Self>::new(self, port, queue)
    }

    fn compose(self) -> CompositionBatch<Self> {
        CompositionBatch::<Self>::new(self)
    }

    /// Get parent batch.
    fn pop(&mut self) -> &mut Self::Parent;
}

/// Public interface implemented by packet batches which manipulate headers.
pub trait HeaderOperations : Batch {
    type Header : EndOffset;
    /// Transform a header field.
    fn transform(self, transformer: Box<FnMut(&mut Self::Header)>) -> TransformBatch<Self::Header, Self> {
        TransformBatch::<Self::Header, Self>::new(self, transformer)
    }

    /// Map over a set of header fields. Map and transform primarily differ in map being immutable. Immutability
    /// provides some optimization opportunities not otherwise available.
    fn map(self, transformer: Box<FnMut(&Self::Header)>) -> MapBatch<Self::Header, Self> {
        MapBatch::<Self::Header, Self>::new(self, transformer)
    }

    /// Rewrite the entire header.
    fn replace(self, template: Self::Header) -> ReplaceBatch<Self::Header, Self> {
        ReplaceBatch::<Self::Header, Self>::new(self, template)
    }

}
