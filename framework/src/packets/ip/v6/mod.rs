use common::Result;
use native::zcsi::MBuf;
use std::fmt;
use std::net::Ipv6Addr;
use packets::{Fixed, Packet, Header, Ethernet};
use packets::ip::{IpPacket, ProtocolNumber};

pub use self::srh::*;

pub mod srh;

/// Common behaviors shared by IPv6 and extension packets
pub trait Ipv6Packet: IpPacket {
}

/*  (From RFC8200 https://tools.ietf.org/html/rfc8200#section-3)
    IPv6 Header Format

    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |Version| Traffic Class |           Flow Label                  |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |         Payload Length        |  Next Header  |   Hop Limit   |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |                                                               |
    +                                                               +
    |                                                               |
    +                         Source Address                        +
    |                                                               |
    +                                                               +
    |                                                               |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |                                                               |
    +                                                               +
    |                                                               |
    +                      Destination Address                      +
    |                                                               |
    +                                                               +
    |                                                               |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

    Version             4-bit Internet Protocol version number = 6.

    Traffic Class       8-bit traffic class field.

    Flow Label          20-bit flow label.

    Payload Length      16-bit unsigned integer.  Length of the IPv6
                        payload, i.e., the rest of the packet following
                        this IPv6 header, in octets.  (Note that any
                        extension headers present are considered part of
                        the payload, i.e., included in the length count.)

    Next Header         8-bit selector.  Identifies the type of header
                        immediately following the IPv6 header.  Uses the
                        same values as the IPv4 Protocol field [RFC-1700
                        et seq.].

    Hop Limit           8-bit unsigned integer.  Decremented by 1 by
                        each node that forwards the packet. The packet
                        is discarded if Hop Limit is decremented to
                        zero.

    Source Address      128-bit address of the originator of the packet.

    Destination Address 128-bit address of the intended recipient of the
                        packet (possibly not the ultimate recipient, if
                        a Routing header is present).
*/

/// IPv6 header
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct Ipv6Header {
    version_to_flow_label: u32,
    payload_len: u16,
    next_header: u8,
    hop_limit: u8,
    src: Ipv6Addr,
    dst: Ipv6Addr,
}

impl Default for Ipv6Header {
    fn default() -> Ipv6Header {
        Ipv6Header {
            version_to_flow_label: u32::to_be(6 << 28),
            payload_len: 0,
            next_header: 0,
            hop_limit: 0,
            src: Ipv6Addr::UNSPECIFIED,
            dst: Ipv6Addr::UNSPECIFIED,
        }
    }
}

impl Header for Ipv6Header {}

/// IPv6 packet
pub struct Ipv6 {
    envelope: Ethernet,
    mbuf: *mut MBuf,
    offset: usize,
    header: *mut Ipv6Header
}

impl Ipv6 {
    #[inline]
    pub fn version(&self) -> u8 {
        // Protocol Version, should always be `6`
        ((u32::from_be(self.header().version_to_flow_label) & 0xf0000000) >> 28) as u8
    }

    #[inline]
    pub fn traffic_class(&self) -> u8 {
        ((u32::from_be(self.header().version_to_flow_label) >> 20) as u8)
    }

    #[inline]
    pub fn set_traffic_class(&mut self, traffic_class: u8) {
        self.header().version_to_flow_label = u32::to_be(
            (u32::from_be(self.header().version_to_flow_label) & 0xf00fffff) | ((traffic_class as u32) << 20),
        )
    }

    #[inline]
    pub fn flow_label(&self) -> u32 {
        u32::from_be(self.header().version_to_flow_label) & 0x0fffff
    }

    #[inline]
    pub fn set_flow_label(&mut self, flow_label: u32) {
        assert!(flow_label <= 0x0fffff);
        self.header().version_to_flow_label = u32::to_be(
            (u32::from_be(self.header().version_to_flow_label) & 0xfff00000) | (flow_label & 0x0fffff)
        )
    }

    #[inline]
    pub fn payload_len(&self) -> u16 {
        u16::from_be(self.header().payload_len)
    }

    #[inline]
    pub fn set_payload_len(&mut self, payload_len: u16) {
        self.header().payload_len = u16::to_be(payload_len)
    }

    #[inline]
    pub fn next_header(&self) -> ProtocolNumber {
        ProtocolNumber::new(self.header().next_header)
    }

    #[inline]
    pub fn set_next_header(&mut self, next_header: ProtocolNumber) {
        self.header().next_header = next_header.0
    }

    #[inline]
    pub fn hop_limit(&self) -> u8 {
        self.header().hop_limit
    }

    #[inline]
    pub fn set_hop_limit(&mut self, hop_limit: u8) {
        self.header().hop_limit = hop_limit;
    }

    #[inline]
    pub fn src(&self) -> Ipv6Addr {
        self.header().src
    }

    #[inline]
    pub fn set_src(&mut self, src: Ipv6Addr) {
        self.header().src = src
    }

    #[inline]
    pub fn dst(&self) -> Ipv6Addr {
        self.header().dst
    }

    #[inline]
    pub fn set_dst(&mut self, dst: Ipv6Addr) {
        self.header().dst = dst
    }
}

impl fmt::Display for Ipv6 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} > {}, version: {}, traffic_class: {}, flow_label: {}, len: {}, next_header: {}, hop_limit: {}",
            self.src(),
            self.dst(),
            self.version(),
            self.traffic_class(),
            self.flow_label(),
            self.payload_len(),
            self.next_header(),
            self.hop_limit()
        )
    }
}

impl Packet for Ipv6 {
    type Header = Ipv6Header;
    type Envelope = Ethernet;

    #[inline]
    fn from_packet(envelope: Self::Envelope,
                   mbuf: *mut MBuf,
                   offset: usize,
                   header: *mut Self::Header) -> Result<Self> {
        Ok(Ipv6 {
            envelope,
            mbuf,
            offset,
            header
        })
    }

    #[inline]
    fn envelope(&self) -> &Self::Envelope {
        &self.envelope
    }

    #[inline]
    fn mbuf(&self) -> *mut MBuf {
        self.mbuf
    }

    #[inline]
    fn offset(&self) -> usize {
        self.offset
    }

    #[inline]
    fn header(&self) -> &mut Self::Header {
        unsafe { &mut (*self.header) }
    }

    #[inline]
    fn header_len(&self) -> usize {
        Self::Header::size()
    }
}

impl IpPacket for Ipv6 {
    fn next_proto(&self) -> ProtocolNumber {
        self.next_header()
    }
}

impl Ipv6Packet for Ipv6 {}

#[cfg(test)]
mod tests {
    use super::*;
    use packets::RawPacket;
    use packets::ip::ProtocolNumbers;
    use dpdk_test;

    #[rustfmt::skip]
    pub const IPV6_PACKET: [u8; 62] = [
        // ** ethernet header
        0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
        0x86, 0xDD,
        // ** IPv6 header
        // version, traffic class, flow label
        0x60, 0x00, 0x00, 0x00,
        // payload length
        0x00, 0x08,
        // next Header
        0x11,
        // hop limit
        0x02,
        // src addr
        0x20, 0x01, 0x0d, 0xb8, 0x85, 0xa3, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        // dst addr
        0x20, 0x01, 0x0d, 0xb8, 0x85, 0xa3, 0x00, 0x00, 0x00, 0x00, 0x8a, 0x2e, 0x03, 0x70, 0x73, 0x34,
        // ** UDP header
        // src port
        0x0d, 0x88,
        // dst port
        0x04, 0x00,
        // length
        0x00, 0x08,
        // checksum
        0x00, 0x00
    ];

    #[test]
    fn size_of_ipv6_header() {
        assert_eq!(40, Ipv6Header::size());
    }

    #[test]
    fn parse_ipv6_packet() {
        dpdk_test! {
            let packet = RawPacket::from_bytes(&IPV6_PACKET).unwrap();
            let ethernet = packet.parse::<Ethernet>().unwrap();
            let ipv6 = ethernet.parse::<Ipv6>().unwrap();

            assert_eq!(6, ipv6.version());
            assert_eq!(0, ipv6.traffic_class());
            assert_eq!(0, ipv6.flow_label());
            assert_eq!(8, ipv6.payload_len());
            assert_eq!(ProtocolNumbers::Udp, ipv6.next_header());
            assert_eq!(2, ipv6.hop_limit());
            assert_eq!("2001:db8:85a3::1", ipv6.src().to_string());
            assert_eq!("2001:db8:85a3::8a2e:370:7334", ipv6.dst().to_string());
        }
    }
}
