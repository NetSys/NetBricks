use failure::Error;
use interface::PacketTx;
use native::zcsi::MBuf;
use packets::Packet;
use std::collections::HashMap;

pub use self::filter_batch::*;
pub use self::filtermap_batch::*;
pub use self::foreach_batch::*;
pub use self::groupby_batch::*;
pub use self::map_batch::*;
pub use self::queue_batch::*;
pub use self::receive_batch::*;
pub use self::send_batch::*;
pub use self::emit_batch::*;

mod filter_batch;
mod filtermap_batch;
mod foreach_batch;
mod groupby_batch;
mod map_batch;
mod queue_batch;
mod receive_batch;
mod send_batch;
mod emit_batch;

/// Error when processing packets
#[derive(Debug)]
pub enum PacketError {
    /// Processing is complete; emit the packet
    Emit(*mut MBuf),
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
    #[inline]
    fn filter<P>(self, predicate: P) -> FilterBatch<Self, P>
    where
        P: FnMut(&Self::Item) -> bool,
        Self: Sized,
    {
        FilterBatch::new(self, predicate)
    }

    ///
    #[inline]
    fn filter_map<T: Packet, F>(self, f: F) -> FilterMapBatch<Self, T, F>
    where
        F: FnMut(Self::Item) -> Result<Option<T>, Error>,
        Self: Sized,
    {
        FilterMapBatch::new(self, f)
    }

    /// Appends a map operator to the end of the pipeline
    #[inline]
    fn map<T: Packet, M>(self, map: M) -> MapBatch<Self, T, M>
    where
        M: FnMut(Self::Item) -> Result<T, Error>,
        Self: Sized,
    {
        MapBatch::new(self, map)
    }

    /// Appends a for_each operator to the end of the pipeline
    ///
    /// Use for side-effects on packets, meaning the packets will not be
    /// transformed byte-wise.
    #[inline]
    fn for_each<F>(self, fun: F) -> ForEachBatch<Self, F>
    where
        F: FnMut(&Self::Item) -> Result<(), Error>,
        Self: Sized,
    {
        ForEachBatch::new(self, fun)
    }

    /// Appends a group_by operator to the end of the pipeline
    ///
    /// * `selector` - a function that receives a reference to `B::Item` and
    /// evaluates to a discriminator value. The source batch will be split
    /// into subgroups based on this value.
    ///
    /// * `composer` - a function that composes the pipelines for the subgroups
    /// based on the discriminator values.
    ///
    /// # Example
    ///
    /// ```
    /// let batch = batch.group_by(
    ///     |packet| packet.protocol(),
    ///     |groups| {
    ///         compose!(
    ///             groups,
    ///             ProtocolNumbers::Tcp => |group| {
    ///                 group.map(handle_tcp)
    ///             },
    ///             ProtocolNumbers::Udp => |group| {
    ///                 group.map(handle_udp)
    ///             }
    ///         )
    ///     }
    /// );
    /// ```
    #[inline]
    fn group_by<K, S, C>(self, selector: S, composer: C) -> GroupByBatch<Self, K, S>
    where
        K: Eq + Clone + std::hash::Hash,
        S: FnMut(&Self::Item) -> K,
        C: FnOnce(&mut HashMap<Option<K>, Box<PipelineBuilder<Self::Item>>>) -> (),
        Self: Sized,
    {
        GroupByBatch::new(self, selector, composer)
    }

    /// Appends a emit operator to the end of the pipeline
    ///
    /// Use when processing is complete and no further modifications are necessary.
    /// Any further operators will have no effect on packets that have been through
    /// the emit operator. Emit the packet as-is.
    fn emit(self) -> EmitBatch<Self>
    where
        Self: Sized,
    {
        EmitBatch::new(self)
    }

    /// Appends a send operator to the end of the pipeline
    ///
    /// Send marks the end of the pipeline. No more operators can be
    /// appended after send.
    #[inline]
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
    use compose;
    use dpdk_test;
    use packets::ip::v4::Ipv4;
    use packets::ip::ProtocolNumbers;
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
    fn filter_map_operator() {
        use packets::icmp::v4::tests::ICMPV4_PACKET;
        use packets::udp::tests::UDP_PACKET;

        dpdk_test! {
            let (producer, batch) = single_threaded_batch::<RawPacket>(2);
            let mut batch = batch.filter_map(|p| {
                let v4 = p.parse::<Ethernet>()?.parse::<Ipv4>()?;
                if v4.protocol() == ProtocolNumbers::Udp {
                    let mut eth  = v4.deparse();
                    eth.swap_addresses();
                    Ok(Some(eth.parse::<Ipv4>()?))
                } else {
                    Ok(None)
                }
            });


            producer.enqueue(RawPacket::from_bytes(&UDP_PACKET).unwrap());
            producer.enqueue(RawPacket::from_bytes(&ICMPV4_PACKET).unwrap());

            let udp_packet = RawPacket::from_bytes(&UDP_PACKET).unwrap();
            let ethernet = udp_packet.parse::<Ethernet>().unwrap();
            let p1 = batch.next().unwrap().unwrap();
            assert_eq!(ProtocolNumbers::Udp, p1.protocol());
            let p1v4 = p1.deparse();
            assert_eq!(ethernet.dst(), p1v4.src());
            assert_eq!(ethernet.src(), p1v4.dst());

            assert!(batch.next().unwrap().is_err());
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

    #[test]
    fn group_by_operator() {
        use packets::icmp::v4::tests::ICMPV4_PACKET;
        use packets::tcp::tests::TCP_PACKET;
        use packets::udp::tests::UDP_PACKET;

        dpdk_test! {
            let (producer, batch) = single_threaded_batch::<RawPacket>(2);

            let mut batch = batch
                .map(|p| p.parse::<Ethernet>()?.parse::<Ipv4>())
                .group_by(
                    |p| p.protocol(),
                    |groups| {
                        compose!(
                            groups,
                            ProtocolNumbers::Tcp => |group| {
                                group.map(|mut p| {
                                    p.set_ttl(1);
                                    Ok(p)
                                })
                            },
                            ProtocolNumbers::Udp => |group| {
                                group.map(|mut p| {
                                    p.set_ttl(2);
                                    Ok(p)
                                })
                            },
                            _ => |group| {
                                group.filter(|_| {
                                    false
                                })
                            }
                        );
                    }
                );

            producer.enqueue(RawPacket::from_bytes(&TCP_PACKET).unwrap());
            producer.enqueue(RawPacket::from_bytes(&UDP_PACKET).unwrap());
            producer.enqueue(RawPacket::from_bytes(&ICMPV4_PACKET).unwrap());

            let p1 = batch.next().unwrap().unwrap();
            assert_eq!(1, p1.ttl());
            let p2 = batch.next().unwrap().unwrap();
            assert_eq!(2, p2.ttl());
            assert!(batch.next().unwrap().is_err());
        }
    }

    #[test]
    fn emit_operator() {
        use packets::tcp::tests::TCP_PACKET;
        use packets::ethernet::MacAddr;

        dpdk_test! {
            let (producer, batch) = single_threaded_batch::<RawPacket>(1);
            let mut batch = batch
                .map(|p| p.parse::<Ethernet>())
                .map(|mut e| {
                    // ff:ff:ff:ff:ff:ff
                    e.set_src(MacAddr::new(255, 255, 255, 255, 255, 255));
                    Ok(e)
                })
                .emit()
                .map(|mut e| {
                    e.set_src(MacAddr::new(0x12, 0x34, 0x56, 0xAB, 0xCD, 0xEF));
                    Ok(e)
                });
            producer.enqueue(RawPacket::from_bytes(&TCP_PACKET).unwrap());

            if let Err(PacketError::Emit(mbuf)) = batch.next().unwrap() {
                let eth = RawPacket::from_mbuf(mbuf).parse::<Ethernet>().unwrap();
                assert_eq!("ff:ff:ff:ff:ff:ff", eth.src().to_string());
            } else {
                assert!(false, "Unexpected packet result :(");
            }
        }
    }
}
