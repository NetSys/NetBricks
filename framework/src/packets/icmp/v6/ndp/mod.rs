use packets::icmp::v6::{Icmpv6, Icmpv6Packet, Icmpv6Payload, NdpOptionsIterator};
use packets::ip::v6::Ipv6Packet;
use packets::Packet;

pub mod neighbor_advert;
pub mod neighbor_solicit;
pub mod options;
pub mod router_advert;
pub mod router_solicit;

/// NDP message payload marker
pub trait NdpPayload: Icmpv6Payload {}

/// Common behaviors shared by NDP packets
///
/// NDP packets are also ICMPv6 packets.
pub trait NdpPacket<E: Ipv6Packet, P: NdpPayload>: Icmpv6Packet<E, P> {
    /// Returns an iterator that iterates through the options in the NDP packet
    fn options(&self) -> NdpOptionsIterator;
}

impl<E: Ipv6Packet, P: NdpPayload> NdpPacket<E, P> for Icmpv6<E, P>
where
    Icmpv6<E, P>: Icmpv6Packet<E, P>,
{
    fn options(&self) -> NdpOptionsIterator {
        let mbuf = self.mbuf();
        let offset = self.payload_offset() + P::size();
        NdpOptionsIterator::new(mbuf, offset)
    }
}
