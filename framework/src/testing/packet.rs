//! proptest strategies to generate packets

use crate::packets::ip::v6::Ipv6;
use crate::packets::ip::{IpPacket, ProtocolNumber, ProtocolNumbers};
use crate::packets::{EtherType, EtherTypes, Ethernet, MacAddr, Packet, RawPacket, Tcp};
use proptest::arbitrary::{any, Arbitrary};
use proptest::strategy::{Just, Strategy};
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::net::Ipv6Addr;

/// Enumeration of settable packet fields
#[allow(non_camel_case_types)]
#[derive(Debug, Eq, Hash, PartialEq)]
pub enum field {
    // ethernet
    eth_src,
    eth_dst,
    // ipv6
    ipv6_src,
    ipv6_dst,
    ipv6_dscp,
    ipv6_ecn,
    ipv6_flow_label,
    ipv6_hop_limit,
    // tcp
    tcp_src_port,
    tcp_dst_port,
    tcp_seq_no,
    tcp_ack_no,
    tcp_window,
    tcp_urgent_pointer,
    tcp_ns,
    tcp_cwr,
    tcp_ece,
    tcp_urg,
    tcp_ack,
    tcp_psh,
    tcp_rst,
    tcp_syn,
    tcp_fin,
}

/// `HashMap` of packet fields to their corresponding proptest strategy
///
/// Use the `fieldmap!` macro to define fields with their default values.
/// The fields with defaults are fixed to that value. All other fields
/// will use the `Any` strategy to generate random values.
///
/// # Example
///
/// ```
/// let map = fieldmap! {
///     field::ipv6_dst_port => 80,
///     field::tcp_syn => true,
/// }
/// ```
///
/// When converting default value to proptest strategy, if the type of the
/// value does not match the field type, the conversion will `panic`.
pub struct StrategyMap(HashMap<field, Box<Any>>);

impl StrategyMap {
    pub fn new(inner: HashMap<field, Box<Any>>) -> Self {
        StrategyMap(inner)
    }

    fn get<T: Arbitrary + Clone + 'static>(&self, key: &field) -> impl Strategy<Value = T> {
        if let Some(ref v) = self.0.get(key) {
            let v = v
                .downcast_ref::<T>()
                .unwrap_or_else(|| panic!("value doesn't match type for field '{:?}'", key));
            Just(v.clone()).boxed()
        } else {
            any::<T>().boxed()
        }
    }

    fn bool(&self, key: &field) -> impl Strategy<Value = bool> {
        self.get::<bool>(key)
    }

    fn u8(&self, key: &field) -> impl Strategy<Value = u8> {
        self.get::<u8>(key)
    }

    fn u16(&self, key: &field) -> impl Strategy<Value = u16> {
        self.get::<u16>(key)
    }

    fn u32(&self, key: &field) -> impl Strategy<Value = u32> {
        self.get::<u32>(key)
    }

    fn mac_addr(&self, key: &field) -> impl Strategy<Value = MacAddr> {
        self.get::<MacAddr>(key)
    }

    fn ipv6_addr(&self, key: &field) -> impl Strategy<Value = Ipv6Addr> {
        self.get::<Ipv6Addr>(key)
    }
}

#[macro_export]
macro_rules! fieldmap {
    ($($key:expr => $value:expr),*) => {
        {
            #[allow(unused_mut)]
            let mut hashmap = ::std::collections::HashMap::<$crate::testing::field, Box<::std::any::Any>>::new();
            $(
                hashmap.insert($key, Box::new($value));
            )*
            $crate::testing::StrategyMap::new(hashmap)
        }
    };
}

fn ethernet(ether_type: EtherType, map: &StrategyMap) -> impl Strategy<Value = Ethernet> {
    (map.mac_addr(&field::eth_src), map.mac_addr(&field::eth_dst)).prop_map(move |(src, dst)| {
        let packet = RawPacket::new().unwrap();
        let mut packet = packet.push::<Ethernet>().unwrap();
        packet.set_src(src);
        packet.set_dst(dst);
        packet.set_ether_type(ether_type);
        packet
    })
}

fn ipv6(next_header: ProtocolNumber, map: &StrategyMap) -> impl Strategy<Value = Ipv6> {
    (
        ethernet(EtherTypes::Ipv6, map),
        map.ipv6_addr(&field::ipv6_src),
        map.ipv6_addr(&field::ipv6_dst),
        map.u8(&field::ipv6_ecn),
        map.u8(&field::ipv6_dscp),
        map.u32(&field::ipv6_flow_label),
        map.u8(&field::ipv6_hop_limit),
    )
        .prop_map(
            move |(packet, src, dst, ecn, dscp, flow_label, hop_limit)| {
                let mut packet = packet.push::<Ipv6>().unwrap();
                packet.set_src(src);
                packet.set_dst(dst);
                packet.set_ecn(ecn);
                packet.set_dscp(dscp);
                packet.set_flow_label(flow_label);
                packet.set_hop_limit(hop_limit);
                packet.set_next_header(next_header);
                packet
            },
        )
}

fn tcp<E: Debug + IpPacket>(
    envelope: impl Strategy<Value = E>,
    map: &StrategyMap,
) -> impl Strategy<Value = RawPacket> {
    (
        envelope,
        map.u16(&field::tcp_src_port),
        map.u16(&field::tcp_dst_port),
        map.u32(&field::tcp_seq_no),
        map.u32(&field::tcp_ack_no),
        map.u16(&field::tcp_window),
        map.u16(&field::tcp_urgent_pointer),
        // proptest tuple has a limit of 10, this hack gets around that limitation
        (
            map.bool(&field::tcp_ns),
            map.bool(&field::tcp_cwr),
            map.bool(&field::tcp_ece),
            map.bool(&field::tcp_urg),
            map.bool(&field::tcp_ack),
            map.bool(&field::tcp_psh),
            map.bool(&field::tcp_rst),
            map.bool(&field::tcp_syn),
            map.bool(&field::tcp_fin),
        ),
    )
        .prop_map(
            |(
                packet,
                src_port,
                dst_port,
                seq_no,
                ack_no,
                window,
                urgent_pointer,
                (ns, cwr, ece, urg, ack, psh, rst, syn, fin),
            )| {
                let mut packet = packet.push::<Tcp<E>>().unwrap();
                packet.set_src_port(src_port);
                packet.set_dst_port(dst_port);
                packet.set_seq_no(seq_no);
                packet.set_ack_no(ack_no);
                packet.set_window(window);
                packet.set_urgent_pointer(urgent_pointer);
                if ns {
                    packet.set_ns();
                }
                if cwr {
                    packet.set_cwr();
                }
                if ece {
                    packet.set_ece();
                }
                if urg {
                    packet.set_urg();
                }
                if ack {
                    packet.set_ack();
                }
                if psh {
                    packet.set_psh();
                }
                if rst {
                    packet.set_rst();
                }
                if syn {
                    packet.set_syn();
                }
                if fin {
                    packet.set_fin();
                }
                packet.cascade();
                packet.reset()
            },
        )
}

/// Returns a strategy to generate IPv6 TCP packets
///
/// All settable fields are randomly generated. Some field values are implied
/// in order for the packet to be internally consistent. For example,
/// `ether_type` is always `EtherTypes::Ipv6` and `next_header` is always
/// `ProtocolNumbers::Tcp`.
pub fn v6_tcp() -> impl Strategy<Value = RawPacket> {
    v6_tcp_with(fieldmap! {})
}

/// Returns a strategy to generate IPv6 TCP packets
///
/// Similar to `v6_tcp`. Some fields can be explicitly set through `fieldmap!`.
/// All other fields are randomly generated. See the `field` enumfor a list
/// of fields that can be set explicitly.
///
/// # Example
///
/// ```
/// dpdk_test! {
///     let mut runner = TestRunner::default();
///     runner.run(
///         &(v6_tcp(fieldmap! {
///             field::eth_src => MacAddr::new(1, 2, 3, 4, 5, 6),
///             field::tcp_dst_port => 80,
///         })),
///         |packet| {
///             let packet = packet.parse::<Ethernet>().unwrap();
///             assert_eq!(EtherTypes::Ipv6, packet.ether_type());
///             Ok(())
///         }
///     ).unwrap();
/// }
/// ```
pub fn v6_tcp_with(map: StrategyMap) -> impl Strategy<Value = RawPacket> {
    let envelope = ipv6(ProtocolNumbers::Tcp, &map);
    tcp(envelope, &map)
}
