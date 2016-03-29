use self::act::Act;
use self::iterator::BatchIterator;
pub use self::parsed_batch::ParsedBatch;
pub use self::deparsed_batch::DeparsedBatch;
pub use self::transform_batch::TransformBatch;
pub use self::apply_batch::ReplaceBatch;
pub use self::receive_batch::ReceiveBatch;
pub use self::send_batch::SendBatch;
pub use self::map_batch::MapBatch;
use self::map_batch::MapFn;
pub use self::composition_batch::CompositionBatch;
pub use self::filter_batch::FilterBatch;
use self::filter_batch::FilterFn;
pub use self::merge_batch::MergeBatch;
pub use self::context_batch::ContextBatch;
pub use self::reset_parse::ResetParsingBatch;
use super::interface::EndOffset;
use super::pmd::*;
use std::any::Any;

#[macro_use]
mod macros;

mod act;
mod apply_batch;
mod composition_batch;
mod context_batch;
mod deparsed_batch;
mod filter_batch;
mod iterator;
mod map_batch;
mod merge_batch;
mod packet_batch;
mod parsed_batch;
mod reset_parse;
mod receive_batch;
mod send_batch;
mod transform_batch;

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
        where Self: Sized
    {
        ParsedBatch::<T, Self>::new(self)
    }

    /// Send this batch out a particular port and queue.
    fn send(self, port: PmdPort, queue: i32) -> SendBatch<Self>
        where Self: Sized
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
        where Self: Sized + 'static
    {
        CompositionBatch::new(box self)
    }

    /// Add context (i.e., a per packet structure) that can be used during computation.
    fn context<T>(self) -> ContextBatch<T, Self>
        where Self: Sized,
              T: Any + Default + Clone
    {
        ContextBatch::<T, Self>::new(self)
    }
}

/// Public interface implemented by packet batches which manipulate headers.
pub trait HeaderOperations : Batch + Sized {
    type Header : EndOffset;
    /// Transform a header field.
    fn transform(self,
                 transformer: Box<FnMut(&mut Self::Header, &mut [u8], Option<&mut Any>)>)
                 -> TransformBatch<Self::Header, Self> {
        TransformBatch::<Self::Header, Self>::new(self, transformer)
    }

    /// Map over a set of header fields. Map and transform primarily differ in map being immutable. Immutability
    /// provides some optimization opportunities not otherwise available.
    fn map(self, transformer: MapFn<Self::Header>) -> MapBatch<Self::Header, Self> {
        MapBatch::<Self::Header, Self>::new(self, transformer)
    }

    /// Rewrite the entire header.
    fn replace(self, template: Self::Header) -> ReplaceBatch<Self::Header, Self> {
        ReplaceBatch::<Self::Header, Self>::new(self, template)
    }

    /// Filter out packets, any packets for which `filter_f` returns false are dropped from the batch.
    fn filter(self, filter_f: FilterFn<Self::Header>) -> FilterBatch<Self::Header, Self> {
        FilterBatch::<Self::Header, Self>::new(self, filter_f)
    }

    /// Reset the packet pointer to 0. This is identical to composition except for using static dispatch.
    fn reset(self) -> ResetParsingBatch<Self>
        where Self: Sized
    {
        ResetParsingBatch::<Self>::new(self)
    }

    /// Deparse, i.e., remove the last patched header. Note the assumption here is that T = the last header parsed
    /// (which we cannot statically enforce since we loose reference to that header).
    fn deparse<T: EndOffset>(self) -> DeparsedBatch<T, Self>
        where Self: Sized
    {
        DeparsedBatch::<T, Self>::new(self)
    }

}
