use packets::icmp::v6::{Icmpv6, Icmpv6Packet, Icmpv6Payload, Icmpv6Type, Icmpv6Types, NdpPayload};
use packets::ip::v6::Ipv6Packet;
use std::fmt;

/*  From https://tools.ietf.org/html/rfc4861#section-4.1
    Router Solicitation Message Format

     0                   1                   2                   3
     0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |     Type      |     Code      |          Checksum             |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |                            Reserved                           |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |   Options ...
    +-+-+-+-+-+-+-+-+-+-+-+-

    Reserved        This field is unused.  It MUST be initialized to
                    zero by the sender and MUST be ignored by the
                    receiver.

   Valid Options:

    Source link-layer address
                    The link-layer address of the sender, if
                    known.  MUST NOT be included if the Source Address
                    is the unspecified address.  Otherwise, it SHOULD
                    be included on link layers that have addresses.
*/

/// NDP router solicitation message
#[derive(Default, Debug)]
#[repr(C, packed)]
pub struct RouterSolicitation {
    reserved: u32,
}

impl Icmpv6Payload for RouterSolicitation {
    #[inline]
    fn msg_type() -> Icmpv6Type {
        Icmpv6Types::RouterSolicitation
    }
}

impl NdpPayload for RouterSolicitation {}

/// NDP router solicitation packet
impl<E: Ipv6Packet> Icmpv6<E, RouterSolicitation> {
    #[inline]
    pub fn reserved(&self) -> u32 {
        u32::from_be(self.payload().reserved)
    }
}

impl<E: Ipv6Packet> fmt::Display for Icmpv6<E, RouterSolicitation> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "type: {}, code: {}, checksum: 0x{:04x}, reserved: {}",
            self.msg_type(),
            self.code(),
            self.checksum(),
            self.reserved()
        )
    }
}

impl<E: Ipv6Packet> Icmpv6Packet<E, RouterSolicitation> for Icmpv6<E, RouterSolicitation> {
    #[inline]
    fn payload(&self) -> &mut RouterSolicitation {
        unsafe { &mut (*self.payload) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dpdk_test;
    use packets::icmp::v6::{Icmpv6Message, Icmpv6Parse, Icmpv6Types};
    use packets::ip::v6::Ipv6;
    use packets::{Ethernet, Fixed, Packet, RawPacket};

    #[rustfmt::skip]
    const ROUTER_SOLICIT_PACKET: [u8; 70] = [
        // ** ethernet header
        0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
        0x86, 0xDD,
        // ** IPv6 header
        0x60, 0x00, 0x00, 0x00,
        // payload length
        0x00, 0x10,
        0x3a,
        0xff,
        0xfe, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xd4, 0xf0, 0x45, 0xff, 0xfe, 0x0c, 0x66, 0x4b,
        0xff, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        // ** ICMPv6 header
        // type
        0x85,
        // code
        0x00,
        // checksum
        0xf5, 0x0c,
        // ** router solicitation message
        // reserved
        0x00, 0x00, 0x00, 0x00,
        // ** source link-layer address option
        0x01, 0x01, 0x70, 0x3a, 0xcb, 0x1b, 0xf9, 0x7a
    ];

    #[test]
    fn size_of_router_solicitation() {
        assert_eq!(4, RouterSolicitation::size());
    }

    #[test]
    fn parse_router_solicitation_packet() {
        dpdk_test! {
            let packet = RawPacket::from_bytes(&ROUTER_SOLICIT_PACKET).unwrap();
            let ethernet = packet.parse::<Ethernet>().unwrap();
            let ipv6 = ethernet.parse::<Ipv6>().unwrap();

            if let Ok(Icmpv6Message::RouterSolicitation(solicit)) = ipv6.parse_icmpv6() {
                assert_eq!(Icmpv6Types::RouterSolicitation, solicit.msg_type());
                assert_eq!(0, solicit.reserved());
            } else {
                panic!("bad packet");
            }
        }
    }
}
