use packets::icmp::v6::{Icmpv6Packet, Icmpv6Payload};

pub mod router_advert;

/// ndp payload marker trait
pub trait NdpPayload: Icmpv6Payload {}

pub trait NdpPacket<T: NdpPayload>: Icmpv6Packet<T> {
}
