pub use self::packet_batch::PacketBatch;
pub use self::parsed_batch::ParsedBatch;
pub use self::transform_batch::TransformBatch;
pub use self::apply_batch::ReplaceBatch;
use super::interface::EndOffset;
use self::iterator::BatchIterator;

#[macro_use]
mod macros;

mod packet_batch;
mod parsed_batch;
mod transform_batch;
mod apply_batch;
mod iterator;

pub trait Act {
    fn act(&mut self) -> &mut Self;
}

/// Public interface implemented by every packet batch type.
pub trait Batch : Sized + BatchIterator + Act {
    type Header : EndOffset;
    type Parent : BatchIterator + Batch + Act;

    fn parse<T: EndOffset>(&mut self) -> ParsedBatch<T, Self> {
        ParsedBatch::<T, Self>::new(self)
    }

    fn transform<'a>(&'a mut self, transformer: &'a Fn(&mut Self::Header)) -> TransformBatch<Self::Header, Self> {
        TransformBatch::<Self::Header, Self>::new(self, transformer)
    }

    fn replace<'a>(&'a mut self, template: &'a Self::Header) -> ReplaceBatch<Self::Header, Self> {
        ReplaceBatch::<Self::Header, Self>::new(self, template)
    }

    fn pop(&mut self) -> &mut Self::Parent;
}
