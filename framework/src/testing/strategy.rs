//! proptest strategies

use crate::packets::ip::{Flow, ProtocolNumbers};
use proptest::arbitrary::any;
use proptest::strategy::{Just, Strategy};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Returns a strategy to generate IPv4 flows
///
/// The IP addresses and ports are random. The protocol can be
/// either TCP, UDP or ICMP.
pub fn v4_flow() -> impl Strategy<Value = Flow> {
    (
        any::<Ipv4Addr>(),
        any::<Ipv4Addr>(),
        any::<u16>(),
        any::<u16>(),
        prop_oneof![
            Just(ProtocolNumbers::Tcp),
            Just(ProtocolNumbers::Udp),
            Just(ProtocolNumbers::Icmpv4),
        ],
    )
        .prop_map(|(src_ip, dst_ip, src_port, dst_port, protocol)| {
            Flow::new(
                IpAddr::V4(src_ip),
                IpAddr::V4(dst_ip),
                src_port,
                dst_port,
                protocol,
            )
        })
}

/// Returns a strategy to generate IPv6 flows
///
/// The IP addresses and ports are random. The protocol can be
/// either TCP, UDP or ICMP.
pub fn v6_flow() -> impl Strategy<Value = Flow> {
    (
        any::<Ipv6Addr>(),
        any::<Ipv6Addr>(),
        any::<u16>(),
        any::<u16>(),
        prop_oneof![
            Just(ProtocolNumbers::Tcp),
            Just(ProtocolNumbers::Udp),
            Just(ProtocolNumbers::Icmpv6),
        ],
    )
        .prop_map(|(src_ip, dst_ip, src_port, dst_port, protocol)| {
            Flow::new(
                IpAddr::V6(src_ip),
                IpAddr::V6(dst_ip),
                src_port,
                dst_port,
                protocol,
            )
        })
}
