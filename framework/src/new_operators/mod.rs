use failure::Error;
use interface::PacketTx;
use native::zcsi::MBuf;
use packets::Packet;

pub use self::filter_batch::*;
pub use self::foreach_batch::*;
pub use self::map_batch::*;
pub use self::queue_batch::*;
pub use self::receive_batch::*;
pub use self::send_batch::*;

mod filter_batch;
mod foreach_batch;
mod map_batch;
mod queue_batch;
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
pub trait Batch: Sized {
    /// The packet type
    type Item: Packet;

    /// Returns the next packet in the batch
    fn next(&mut self) -> Option<Result<Self::Item, PacketError>>;

    /// Receives a new batch
    fn receive(&mut self);

    /// Appends a filter operator to the end of the pipeline
    fn filter<P: FnMut(&Self::Item) -> bool>(self, predicate: P) -> FilterBatch<Self, P> {
        FilterBatch::new(self, predicate)
    }

    /// Appends a map operator to the end of the pipeline
    fn map<T: Packet, M: FnMut(Self::Item) -> Result<T, Error>>(
        self,
        map: M,
    ) -> MapBatch<Self, T, M> {
        MapBatch::new(self, map)
    }

    /// Appends a for_each operator to the end of the pipeline
    ///
    /// Use for side-effects on packets, meaning the packets will not be
    /// transformed byte-wise.
    fn for_each<F: FnMut(&Self::Item) -> Result<(), Error>>(self, fun: F) -> ForEachBatch<Self, F> {
        ForEachBatch::new(self, fun)
    }

    /// Appends a send operator to the end of the pipeline
    ///
    /// Send marks the end of the pipeline. No more operators can be
    /// appended after send.
    fn send<Tx: PacketTx>(self, port: Tx) -> SendBatch<Self, Tx> {
        SendBatch::new(self, port)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use dpdk_test;
    use packets::{EtherTypes, Ethernet, RawPacket};

    #[test]
    fn filter_operator() {
        use packets::udp::tests::UDP_PACKET;

        dpdk_test! {
            let (producer, batch) = single_threaded_batch::<RawPacket>(1);
            let mut batch = batch.filter(|_| false);
            producer.enqueue(RawPacket::from_bytes(&UDP_PACKET).unwrap());

            assert!(batch.next().unwrap().is_err());
        }
    }

    #[test]
    fn map_operator() {
        use packets::udp::tests::UDP_PACKET;

        dpdk_test! {
            let (producer, batch) = single_threaded_batch::<RawPacket>(1);
            let mut batch = batch.map(|p| p.parse::<Ethernet>());
            producer.enqueue(RawPacket::from_bytes(&UDP_PACKET).unwrap());

            let packet = batch.next().unwrap().unwrap();
            assert_eq!(EtherTypes::Ipv4, packet.ether_type())
        }
    }

    #[test]
    fn for_each_operator() {
        use packets::udp::tests::UDP_PACKET;

        dpdk_test! {
            let mut invoked = 0;

            {
                let (producer, batch) = single_threaded_batch::<RawPacket>(1);
                let mut batch = batch.for_each(|_| {
                    invoked += 1;
                    Ok(())
                });
                producer.enqueue(RawPacket::from_bytes(&UDP_PACKET).unwrap());

                let _ = batch.next();
            }

            assert_eq!(1, invoked);
        }
    }
}
