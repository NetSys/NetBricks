pub use self::packet_batch::packet_ptr;
pub use self::packet_batch::PacketBatch;
pub use self::packet_batch::add_to_batch;
pub use self::packet_batch::consumed_batch;
pub use self::parsed_batch::ParsedBatch;
pub use self::transform_batch::TransformBatch;
pub use self::apply_batch::ReplaceBatch;

#[macro_use]
mod macros;

mod packet_batch;
mod parsed_batch;
mod transform_batch;
mod apply_batch;
mod internal_iface;

pub trait Act {
    fn act(&mut self) -> &mut Self;
}
