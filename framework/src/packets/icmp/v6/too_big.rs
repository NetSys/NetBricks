use packets::icmp::v6::{Icmpv6, Icmpv6Packet, Icmpv6Payload, Icmpv6Type, Icmpv6Types};
use packets::ip::v6::Ipv6Packet;
use packets::{buffer, EthernetHeader, Fixed, Packet};
use std::fmt;

/*  From https://tools.ietf.org/html/rfc4443#section-3.2
    Packet Too Big Message

     0                   1                   2                   3
     0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |     Type      |     Code      |          Checksum             |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |                             MTU                               |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |                    As much of invoking packet                 |
    +               as possible without the ICMPv6 packet           +
    |               exceeding the minimum IPv6 MTU [IPv6]           |

    MTU            The Maximum Transmission Unit of the next-hop link.
*/

/// Packet too big message
#[derive(Default, Debug)]
#[repr(C, packed)]
pub struct PacketTooBig {
    mtu: u32,
}

impl Icmpv6Payload for PacketTooBig {
    fn msg_type() -> Icmpv6Type {
        Icmpv6Types::PacketTooBig
    }
}

impl<E: Ipv6Packet> Icmpv6<E, PacketTooBig> {
    #[inline]
    pub fn mtu(&self) -> u32 {
        u32::from_be(self.payload().mtu)
    }

    #[inline]
    pub fn set_mtu(&mut self, mtu: u32) {
        self.payload_mut().mtu = u32::to_be(mtu);
    }
}

impl<E: Ipv6Packet> fmt::Display for Icmpv6<E, PacketTooBig> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "type: {}, code: {}, checksum: 0x{:04x}, mtu: {}",
            self.msg_type(),
            self.code(),
            self.checksum(),
            self.mtu()
        )
    }
}

impl<E: Ipv6Packet> Packet for Icmpv6<E, PacketTooBig> {
    #[inline]
    fn cascade(&mut self) {
        // assuming inside an ethernet frame
        let max_len = self.mtu() as usize + EthernetHeader::size();
        // only err if nothing to trim, ignore the result
        let _ = buffer::trim(self.mbuf(), max_len);

        self.compute_checksum();
        self.envelope_mut().cascade();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_of_packet_too_big() {
        assert_eq!(4, PacketTooBig::size());
    }
}
