use failure::Error;
use interface::PacketTx;
use native::zcsi::MBuf;
use packets::Packet;

pub use self::filter_batch::*;
pub use self::map_batch::*;
#[cfg(test)]
pub use self::packet_batch::*;
pub use self::receive_batch::*;
pub use self::send_batch::*;

mod filter_batch;
mod map_batch;
#[cfg(test)]
mod packet_batch;
mod receive_batch;
mod send_batch;

/// Error when processing packets
#[derive(Debug)]
pub enum PacketError {
    /// The packet is intentionally dropped
    Drop(*mut MBuf),
    /// The packet is aborted due to an error
    Abort(*mut MBuf, Error),
}

/// Common behavior for a batch of packets
pub trait Batch {
    /// The packet type
    type Item: Packet;

    /// Returns the next packet in the batch
    fn next(&mut self) -> Option<Result<Self::Item, PacketError>>;

    /// Receives a new batch
    fn receive(&mut self);

    /// Appends a filter operator to the end of the pipeline
    fn filter<P: Fn(&Self::Item) -> bool>(self, predicate: P) -> FilterBatch<Self, P>
    where
        Self: Sized,
    {
        FilterBatch::new(self, predicate)
    }

    /// Appends a map operator to the end of the pipeline
    fn map<T: Packet, M: Fn(Self::Item) -> Result<T, Error>>(self, map: M) -> MapBatch<Self, T, M>
    where
        Self: Sized,
    {
        MapBatch::new(self, map)
    }

    /// Appends a send operator to the end of the pipeline
    ///
    /// Send marks the end of the pipeline. No more operators can be
    /// appended after send.
    fn send<Tx: PacketTx>(self, port: Tx) -> SendBatch<Self, Tx>
    where
        Self: Sized,
    {
        SendBatch::new(self, port)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use dpdk_test;
    use packets::{EtherTypes, Ethernet};

    #[test]
    fn filter_operator() {
        use packets::udp::tests::UDP_PACKET;

        dpdk_test! {
            let mut batch = PacketBatch::new(&UDP_PACKET).filter(|_| false);
            assert!(batch.next().unwrap().is_err());
        }
    }

    #[test]
    fn map_operator() {
        use packets::udp::tests::UDP_PACKET;

        dpdk_test! {
            let mut batch = PacketBatch::new(&UDP_PACKET).map(|p| p.parse::<Ethernet>());
            let packet = batch.next().unwrap().unwrap();

            assert_eq!(EtherTypes::Ipv4, packet.ether_type())
        }
    }
}
