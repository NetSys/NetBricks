use self::act::Act;
use self::iterator::BatchIterator;
pub use self::parsed_batch::ParsedBatch;
pub use self::transform_batch::TransformBatch;
pub use self::apply_batch::ReplaceBatch;
pub use self::receive_batch::ReceiveBatch;
pub use self::send_batch::SendBatch;
pub use self::map_batch::MapBatch;
pub use self::composition_batch::CompositionBatch;
pub use self::filter_batch::FilterBatch;
pub use self::merge_batch::MergeBatch;
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
mod filter_batch;
mod merge_batch;

/// Merge a vector of batches into one batch. Currently this just round-robins between merged batches, but in the future
/// the precise batch being processed will be determined by the scheduling policy used.
#[inline]
pub fn merge(batches: Vec<CompositionBatch>) -> MergeBatch {
    MergeBatch::new(batches)
}

/// Public trait implemented by every packet batch type. This trait should be used as a constraint for any functions or
/// places where a Batch type is required.
pub trait Batch : BatchIterator + Act {
    /// Parse the payload as header of type.
    fn parse<T: EndOffset>(self) -> ParsedBatch<T, Self> 
        where Self:Sized
    {
        ParsedBatch::<T, Self>::new(self)
    }

    /// Send this batch out a particular port and queue.
    fn send(self, port: PmdPort, queue: i32) -> SendBatch<Self>
        where Self:Sized
    {
        SendBatch::<Self>::new(self, port, queue)
    }

    /// Erase type information. This is essential to allow different kinds of types to be collected together, as done
    /// for example when merging batches or composing different NFs together.
    ///
    /// # Warning
    /// This causes some performance degradation: operations called through composition batches rely on indirect calls
    /// which affect throughput.
    fn compose(self) -> CompositionBatch
        where Self:Sized + 'static
    {
        CompositionBatch::new(box self)
    }
}

/// Public interface implemented by packet batches which manipulate headers.
pub trait HeaderOperations : Batch + Sized {
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

    /// Filter out packets, any packets for which `filter_f` returns false are dropped from the batch.
    fn filter(self, filter_f: Box<FnMut(&Self::Header) -> bool>) -> FilterBatch<Self::Header, Self> {
        FilterBatch::<Self::Header, Self>::new(self, filter_f)
    }
}
