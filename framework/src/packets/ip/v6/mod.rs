use common::Result;
use native::zcsi::MBuf;
use std::fmt;
use std::net::{IpAddr, Ipv6Addr};
use packets::{buffer, Ethernet, Fixed, Header, Packet};
use packets::ip::{IpPacket, ProtocolNumber, IpAddrMismatchError};

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
    pub fn set_traffic_class(&self, traffic_class: u8) {
        self.header().version_to_flow_label = u32::to_be(
            (u32::from_be(self.header().version_to_flow_label) & 0xf00fffff) | ((traffic_class as u32) << 20),
        );
    }

    #[inline]
    pub fn flow_label(&self) -> u32 {
        u32::from_be(self.header().version_to_flow_label) & 0x0fffff
    }

    #[inline]
    pub fn set_flow_label(&self, flow_label: u32) {
        assert!(flow_label <= 0x0fffff);
        self.header().version_to_flow_label = u32::to_be(
            (u32::from_be(self.header().version_to_flow_label) & 0xfff00000) | (flow_label & 0x0fffff)
        );
    }

    #[inline]
    pub fn payload_len(&self) -> u16 {
        u16::from_be(self.header().payload_len)
    }

    #[inline]
    pub fn set_payload_len(&self, payload_len: u16) {
        self.header().payload_len = u16::to_be(payload_len);
    }

    #[inline]
    pub fn next_header(&self) -> ProtocolNumber {
        ProtocolNumber::new(self.header().next_header)
    }

    #[inline]
    pub fn set_next_header(&self, next_header: ProtocolNumber) {
        self.header().next_header = next_header.0;
    }

    #[inline]
    pub fn hop_limit(&self) -> u8 {
        self.header().hop_limit
    }

    #[inline]
    pub fn set_hop_limit(&self, hop_limit: u8) {
        self.header().hop_limit = hop_limit;
    }

    #[inline]
    pub fn src(&self) -> Ipv6Addr {
        self.header().src
    }

    #[inline]
    fn set_src(&self, src: Ipv6Addr) {
        self.header().src = src;
    }

    #[inline]
    pub fn dst(&self) -> Ipv6Addr {
        self.header().dst
    }

    #[inline]
    fn set_dst(&self, dst: Ipv6Addr) {
        self.header().dst = dst;
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

    #[doc(hidden)]
    #[inline]
    fn do_parse(envelope: Self::Envelope) -> Result<Self> {
        let mbuf = envelope.mbuf();
        let offset = envelope.payload_offset();
        let header = buffer::read_item::<Self::Header>(mbuf, offset)?;

        Ok(Ipv6 {
            envelope,
            mbuf,
            offset,
            header
        })
    }

    #[doc(hidden)]
    #[inline]
    fn do_push(envelope: Self::Envelope) -> Result<Self> {
        let mbuf = envelope.mbuf();
        let offset = envelope.payload_offset();

        buffer::alloc(mbuf, offset, Self::Header::size())?;
        let header = buffer::write_item::<Self::Header>(mbuf, offset, &Default::default())?;

        Ok(Ipv6 {
            envelope,
            mbuf,
            offset,
            header
        })
    }
}

impl IpPacket for Ipv6 {
    #[inline]
    fn next_proto(&self) -> ProtocolNumber {
        self.next_header()
    }

    #[inline]
    fn src(&self) -> IpAddr {
        IpAddr::V6(self.src())
    }

    #[inline]
    fn set_src(&self, src: IpAddr) -> Result<()> {
        match src {
            IpAddr::V6(addr) => {
                self.set_src(addr);
                Ok(())
            },
            _ => Err(IpAddrMismatchError.into())
        }
    }

    #[inline]
    fn dst(&self) -> IpAddr {
        IpAddr::V6(self.dst())
    }

    #[inline]
    fn set_dst(&self, dst: IpAddr) -> Result<()> {
        match dst {
            IpAddr::V6(addr) => {
                self.set_dst(addr);
                Ok(())
            },
            _ => Err(IpAddrMismatchError.into())
        }
    }

    /// Returns the IPv6 pseudo-header sum
    /// https://tools.ietf.org/html/rfc2460#section-8.1
    /// 
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    /// |                                                               |
    /// +                                                               +
    /// |                                                               |
    /// +                         Source Address                        +
    /// |                                                               |
    /// +                                                               +
    /// |                                                               |
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    /// |                                                               |
    /// +                                                               +
    /// |                                                               |
    /// +                      Destination Address                      +
    /// |                                                               |
    /// +                                                               +
    /// |                                                               |
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    /// |                   Upper-Layer Packet Length                   |
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    /// |                      zero                     |  Next Header  |
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    #[inline]
    fn pseudo_header_sum(&self, packet_len: u16, protocol: ProtocolNumber) -> u16 {
        let mut sum =
            self.src().segments().iter().fold(0, |acc, &x| { acc + x as u32 }) +
            self.dst().segments().iter().fold(0, |acc, &x| { acc + x as u32 }) +
            packet_len as u32 +
            protocol.0 as u32;
        
        while sum >> 16 != 0 {
            sum = (sum >> 16) + (sum & 0xFFFF);
        }

        sum as u16
    }
}

impl Ipv6Packet for Ipv6 {}

#[cfg(test)]
pub mod tests {
    use super::*;
    use packets::RawPacket;
    use packets::ip::ProtocolNumbers;
    use dpdk_test;

    #[rustfmt::skip]
    pub const IPV6_PACKET: [u8; 78] = [
        // ** ethernet header
        0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
        0x86, 0xDD,
        // ** IPv6 header
        // version, traffic class, flow label
        0x60, 0x00, 0x00, 0x00,
        // payload length
        0x00, 0x18,
        // next Header
        0x11,
        // hop limit
        0x02,
        // src addr
        0x20, 0x01, 0x0d, 0xb8, 0x85, 0xa3, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        // dst addr
        0x20, 0x01, 0x0d, 0xb8, 0x85, 0xa3, 0x00, 0x00, 0x00, 0x00, 0x8a, 0x2e, 0x03, 0x70, 0x73, 0x34,
        // ** TCP header
        // src_port = 36869, dst_port = 23
        0x90, 0x05, 0x00, 0x17,
        // seq_no = 1913975060
        0x72, 0x14, 0xf1, 0x14,
        // ack_no = 0
        0x00, 0x00, 0x00, 0x00,
        // data_offset = 24, flags = 0x02
        0x60, 0x02,
        // window = 8760, checksum = 0xa92c, urgent = 0
        0x22, 0x38, 0xa9, 0x2c, 0x00, 0x00,
        // options
        0x02, 0x04, 0x05, 0xb4
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
            assert_eq!(24, ipv6.payload_len());
            assert_eq!(ProtocolNumbers::Udp, ipv6.next_header());
            assert_eq!(2, ipv6.hop_limit());
            assert_eq!("2001:db8:85a3::1", ipv6.src().to_string());
            assert_eq!("2001:db8:85a3::8a2e:370:7334", ipv6.dst().to_string());
        }
    }

    #[test]
    fn push_ipv6_packet() {
        dpdk_test! {
            let packet = RawPacket::new().unwrap();
            let ethernet = packet.push::<Ethernet>().unwrap();
            let ipv6 = ethernet.push::<Ipv6>().unwrap();

            assert_eq!(6, ipv6.version());
            assert_eq!(Ipv6Header::size(), ipv6.len());
        }
    }
}
