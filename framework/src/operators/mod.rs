use headers::*;
use interface::*;
use scheduler::Scheduler;
use self::act::Act;
pub use self::add_metadata::AddMetadataBatch;
use self::add_metadata::MetadataFn;

pub use self::composition_batch::CompositionBatch;
pub use self::deparsed_batch::DeparsedBatch;
pub use self::filter_batch::FilterBatch;

use self::filter_batch::FilterFn;
pub use self::group_by::*;
use self::iterator::BatchIterator;
pub use self::map_batch::MapBatch;
use self::map_batch::MapFn;
pub use self::merge_batch::MergeBatch;
pub use self::parsed_batch::ParsedBatch;
pub use self::receive_batch::ReceiveBatch;

pub use self::reset_parse::ResetParsingBatch;
pub use self::restore_header::*;
pub use self::send_batch::SendBatch;
pub use self::transform_batch::TransformBatch;
use self::transform_batch::TransformFn;

#[macro_use]
mod macros;

mod act;
mod composition_batch;
mod deparsed_batch;
mod filter_batch;
mod group_by;
mod iterator;
mod map_batch;
mod merge_batch;
mod packet_batch;
mod parsed_batch;
mod receive_batch;
mod reset_parse;
mod send_batch;
mod transform_batch;
mod restore_header;
mod add_metadata;

/// Merge a vector of batches into one batch. Currently this just round-robins between merged batches, but in the future
/// the precise batch being processed will be determined by the scheduling policy used.
#[inline]
pub fn merge<T: Batch>(batches: Vec<T>) -> MergeBatch<T> {
    MergeBatch::new(batches)
}

/// Public trait implemented by every packet batch type. This trait should be used as a constraint for any functions or
/// places where a Batch type is required. We declare batches as sendable, they cannot be copied but we allow it to be
/// sent to another thread.
pub trait Batch: BatchIterator + Act + Send {
    /// Parse the payload as header of type.
    fn parse<T: EndOffset<PreviousHeader = Self::Header>>(self) -> ParsedBatch<T, Self>
        where Self: Sized
    {
        ParsedBatch::<T, Self>::new(self)
    }

    fn metadata<M: Sized + Send>(self,
                                 generator: MetadataFn<Self::Header, Self::Metadata, M>)
                                 -> AddMetadataBatch<M, Self>
        where Self: Sized
    {
        AddMetadataBatch::new(self, generator)
    }

    /// Send this batch out a particular port and queue.
    fn send<Port: PacketTx>(self, port: Port) -> SendBatch<Port, Self>
        where Self: Sized
    {
        SendBatch::<Port, Self>::new(self, port)
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
        CompositionBatch::new(self)
    }

    /// Transform a header field.
    fn transform(self, transformer: TransformFn<Self::Header, Self::Metadata>) -> TransformBatch<Self::Header, Self>
        where Self: Sized
    {
        TransformBatch::<Self::Header, Self>::new(self, transformer)
    }

    /// Map over a set of header fields. Map and transform primarily differ in map being immutable. Immutability
    /// provides some optimization opportunities not otherwise available.
    fn map(self, transformer: MapFn<Self::Header, Self::Metadata>) -> MapBatch<Self::Header, Self>
        where Self: Sized
    {
        MapBatch::<Self::Header, Self>::new(self, transformer)
    }

    /// Filter out packets, any packets for which `filter_f` returns false are dropped from the batch.
    fn filter(self, filter_f: FilterFn<Self::Header, Self::Metadata>) -> FilterBatch<Self::Header, Self>
        where Self: Sized
    {
        FilterBatch::<Self::Header, Self>::new(self, filter_f)
    }

    /// Reset the packet pointer to 0. This is identical to composition except for using static dispatch.
    fn reset(self) -> ResetParsingBatch<Self>
        where Self: Sized
    {
        ResetParsingBatch::<Self>::new(self)
    }

    /// Deparse, i.e., remove the last parsed header. Note the assumption here is that T = the last header parsed
    /// (which we cannot statically enforce since we loose reference to that header).
    fn deparse(self) -> DeparsedBatch<Self>
        where Self: Sized
    {
        DeparsedBatch::<Self>::new(self)
    }

    fn group_by<S: Scheduler + Sized>(self,
                                      groups: usize,
                                      group_f: GroupFn<Self::Header, Self::Metadata>,
                                      sched: &mut S)
                                      -> GroupBy<Self::Header, Self>
        where Self: Sized
    {
        GroupBy::<Self::Header, Self>::new(self, groups, group_f, sched)
    }
}
