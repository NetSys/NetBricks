pub use self::packet_batch::PacketBatch;
pub use self::parsed_batch::ParsedBatch;
pub use self::transform_batch::TransformBatch;
pub use self::apply_batch::ReplaceBatch;
pub use self::receive_batch::ReceiveBatch;
pub use self::send_batch::SendBatch;
use super::interface::EndOffset;
use self::iterator::BatchIterator;
use super::pmd::*;
use super::interface::Result;

#[macro_use]
mod macros;

mod packet_batch;
mod parsed_batch;
mod transform_batch;
mod receive_batch;
mod apply_batch;
mod send_batch;
mod iterator;

// FIXME: This should become private.
pub trait Act {
    fn act(&mut self) -> &mut Self;

    /// Notification indicating we are done processing the current batch of packets
    fn done(&mut self) -> &mut Self;

    fn send_queue(&mut self, port: &mut PmdPort, queue: i32) -> Result<u32>;
}

/// Public interface implemented by every packet batch type.
pub trait Batch : Sized + BatchIterator + Act {
    type Header : EndOffset;
    type Parent : BatchIterator + Batch + Act;

    /// Parse the payload as header of type.
    fn parse<T: EndOffset>(&mut self) -> ParsedBatch<T, Self> {
        ParsedBatch::<T, Self>::new(self)
    }

    /// Transform a header field.
    fn transform<'a>(&'a mut self, transformer: &'a mut FnMut(&mut Self::Header)) -> TransformBatch<Self::Header, Self> {
        TransformBatch::<Self::Header, Self>::new(self, transformer)
    }

    /// Rewrite the entire header.
    fn replace<'a>(&'a mut self, template: &'a Self::Header) -> ReplaceBatch<Self::Header, Self> {
        ReplaceBatch::<Self::Header, Self>::new(self, template)
    }

    /// Go back to the parent.
    fn pop(&mut self) -> &mut Self::Parent;

    /// Send this batch out a particular port and queue.
    fn send<'a>(&'a mut self, port: &'a mut PmdPort, queue: i32) -> SendBatch<Self> {
        SendBatch::<Self>::new(self, port, queue)
    }
}
