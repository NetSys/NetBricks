mod srh;

pub use self::srh::*;

use crate::common::Result;
use crate::native::mbuf::MBuf;
use crate::packets::checksum::PseudoHeader;
use crate::packets::ip::{IpAddrMismatchError, IpPacket, ProtocolNumber};
use crate::packets::{buffer, Ethernet, Fixed, Header, Packet};
use std::fmt;
use std::net::{IpAddr, Ipv6Addr};

/// Common behaviors shared by IPv6 and extension packets
pub trait Ipv6Packet: IpPacket {}

/// The minimum IPv6 MTU
///
/// https://tools.ietf.org/html/rfc2460#section-5
pub const IPV6_MIN_MTU: usize = 1280;

/*  From https://tools.ietf.org/html/rfc8200#section-3
    and https://tools.ietf.org/html/rfc3168 (succeeding traffic class)
    IPv6 Header Format

    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |Version|    DSCP_ECN   |           Flow Label                  |
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

    DSCP_ECN:           8-bit Differentiated services (via RFC 2474 ~
                        https://tools.ietf.org/html/rfc2474) enhancements to the
                        Internet protocol are intended to enable scalable
                        service discrimination in the Internet without the need
                        for per-flow state and signaling at every hop.  A
                        variety of services may be built from a small,
                        well-defined set of building blocks which are deployed
                        in network nodes. The services may be either end-to-end
                        or intra-domain; they include both those that can
                        satisfy quantitative performance requirements (e.g.,
                        peak bandwidth) and those based on relative performance
                        (e.g., "class" differentiation).

                        Taking the last two bits, is ECN, the addition of
                        Explicit Congestion Notification to IP; RFC-3168
                        (https://tools.ietf.org/html/rfc3168) covers this in
                        detail. This uses an ECN field in the IP header with two
                        bits, making four ECN codepoints, '00' to '11'.  The
                        ECN-Capable Transport (ECT) codepoints '10' and '01' are
                        set by the data sender to indicate that the end-points
                        of the transport protocol are ECN-capable; we call them
                        ECT(0) and ECT(1) respectively.  The phrase "the ECT
                        codepoint" in this documents refers to either of the two
                        ECT codepoints.  Routers treat the ECT(0) and ECT(1)
                        codepoints as equivalent.  Senders are free to use
                        either the ECT(0) or the ECT(1) codepoint to indicate
                        ECT, on a packet-by-packet basis.

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

// Masks
const DSCP: u32 = 0x0fc0_0000;
const ECN: u32 = 0x0030_0000;
const FLOW: u32 = 0xfffff;

/// IPv6 header
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct Ipv6Header {
    version_to_flow_label: u32,
    payload_length: u16,
    next_header: u8,
    hop_limit: u8,
    src: Ipv6Addr,
    dst: Ipv6Addr,
}

impl Default for Ipv6Header {
    fn default() -> Ipv6Header {
        Ipv6Header {
            version_to_flow_label: u32::to_be(6 << 28),
            payload_length: 0,
            next_header: 0,
            hop_limit: 0,
            src: Ipv6Addr::UNSPECIFIED,
            dst: Ipv6Addr::UNSPECIFIED,
        }
    }
}

impl Header for Ipv6Header {}

/// IPv6 packet
#[derive(Debug)]
pub struct Ipv6 {
    envelope: Ethernet,
    mbuf: *mut MBuf,
    offset: usize,
    header: *mut Ipv6Header,
}

impl Ipv6 {
    #[inline]
    pub fn version(&self) -> u8 {
        // Protocol Version, should always be `6`
        ((u32::from_be(self.header().version_to_flow_label) & 0xf000_0000) >> 28) as u8
    }

    #[inline]
    pub fn dscp(&self) -> u8 {
        ((u32::from_be(self.header().version_to_flow_label) & DSCP) >> 22) as u8
    }

    #[inline]
    pub fn set_dscp(&mut self, dscp: u8) {
        self.header_mut().version_to_flow_label = u32::to_be(
            (u32::from_be(self.header().version_to_flow_label) & !DSCP)
                | ((u32::from(dscp) << 22) & DSCP),
        );
    }

    #[inline]
    pub fn ecn(&self) -> u8 {
        ((u32::from_be(self.header().version_to_flow_label) & ECN) >> 20) as u8
    }

    #[inline]
    pub fn set_ecn(&mut self, ecn: u8) {
        self.header_mut().version_to_flow_label = u32::to_be(
            (u32::from_be(self.header().version_to_flow_label) & !ECN)
                | ((u32::from(ecn) << 20) & ECN),
        );
    }

    #[inline]
    pub fn flow_label(&self) -> u32 {
        u32::from_be(self.header().version_to_flow_label) & FLOW
    }

    #[inline]
    pub fn set_flow_label(&mut self, flow_label: u32) {
        self.header_mut().version_to_flow_label = u32::to_be(
            (u32::from_be(self.header().version_to_flow_label) & !FLOW) | (flow_label & FLOW),
        );
    }

    #[inline]
    pub fn payload_length(&self) -> u16 {
        u16::from_be(self.header().payload_length)
    }

    #[inline]
    fn set_payload_length(&mut self, payload_length: u16) {
        self.header_mut().payload_length = u16::to_be(payload_length);
    }

    #[inline]
    pub fn next_header(&self) -> ProtocolNumber {
        ProtocolNumber::new(self.header().next_header)
    }

    #[inline]
    pub fn set_next_header(&mut self, next_header: ProtocolNumber) {
        self.header_mut().next_header = next_header.0;
    }

    #[inline]
    pub fn hop_limit(&self) -> u8 {
        self.header().hop_limit
    }

    #[inline]
    pub fn set_hop_limit(&mut self, hop_limit: u8) {
        self.header_mut().hop_limit = hop_limit;
    }

    #[inline]
    pub fn src(&self) -> Ipv6Addr {
        self.header().src
    }

    #[inline]
    pub fn set_src(&mut self, src: Ipv6Addr) {
        self.header_mut().src = src;
    }

    #[inline]
    pub fn dst(&self) -> Ipv6Addr {
        self.header().dst
    }

    #[inline]
    pub fn set_dst(&mut self, dst: Ipv6Addr) {
        self.header_mut().dst = dst;
    }
}

impl fmt::Display for Ipv6 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} > {}, version: {}, dscp: {}, ecn: {}, flow_label: {}, len: {}, next_header: {}, hop_limit: {}",
            self.src(),
            self.dst(),
            self.version(),
            self.dscp(),
            self.ecn(),
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
    fn envelope_mut(&mut self) -> &mut Self::Envelope {
        &mut self.envelope
    }

    #[doc(hidden)]
    #[inline]
    fn mbuf(&self) -> *mut MBuf {
        self.mbuf
    }

    #[inline]
    fn offset(&self) -> usize {
        self.offset
    }

    #[doc(hidden)]
    #[inline]
    fn header(&self) -> &Self::Header {
        unsafe { &(*self.header) }
    }

    #[doc(hidden)]
    #[inline]
    fn header_mut(&mut self) -> &mut Self::Header {
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
            header,
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
            header,
        })
    }

    #[inline]
    fn remove(self) -> Result<Self::Envelope> {
        buffer::dealloc(self.mbuf, self.offset, self.header_len())?;
        Ok(self.envelope)
    }

    #[inline]
    fn cascade(&mut self) {
        let len = self.payload_len() as u16;
        self.set_payload_length(len);
        self.envelope_mut().cascade();
    }

    #[inline]
    fn deparse(self) -> Self::Envelope {
        self.envelope
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
    fn set_src(&mut self, src: IpAddr) -> Result<()> {
        match src {
            IpAddr::V6(addr) => {
                self.set_src(addr);
                Ok(())
            }
            _ => Err(IpAddrMismatchError.into()),
        }
    }

    #[inline]
    fn dst(&self) -> IpAddr {
        IpAddr::V6(self.dst())
    }

    #[inline]
    fn set_dst(&mut self, dst: IpAddr) -> Result<()> {
        match dst {
            IpAddr::V6(addr) => {
                self.set_dst(addr);
                Ok(())
            }
            _ => Err(IpAddrMismatchError.into()),
        }
    }

    #[inline]
    fn pseudo_header(&self, packet_len: u16, protocol: ProtocolNumber) -> PseudoHeader {
        PseudoHeader::V6 {
            src: self.src(),
            dst: self.dst(),
            packet_len,
            protocol,
        }
    }
}

impl Ipv6Packet for Ipv6 {}

#[cfg(test)]
#[rustfmt::skip]
pub const IPV6_PACKET: [u8; 78] = [
    // ** ethernet header
    0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
    0x86, 0xDD,
    // ** IPv6 header
    // version, dscp, ecn, flow label
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packets::ip::ProtocolNumbers;
    use crate::packets::RawPacket;

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
            assert_eq!(0, ipv6.dscp());
            assert_eq!(0, ipv6.ecn());
            assert_eq!(0, ipv6.flow_label());
            assert_eq!(24, ipv6.payload_len());
            assert_eq!(ProtocolNumbers::Udp, ipv6.next_header());
            assert_eq!(2, ipv6.hop_limit());
            assert_eq!("2001:db8:85a3::1", ipv6.src().to_string());
            assert_eq!("2001:db8:85a3::8a2e:370:7334", ipv6.dst().to_string());
        }
    }

    #[test]
    fn parse_ipv6_setter_checks() {
        dpdk_test! {
            let packet = RawPacket::from_bytes(&IPV6_PACKET).unwrap();
            let ethernet = packet.parse::<Ethernet>().unwrap();
            let mut ipv6 = ethernet.parse::<Ipv6>().unwrap();

            assert_eq!(6, ipv6.version());
            assert_eq!(0, ipv6.dscp());
            assert_eq!(0, ipv6.ecn());
            assert_eq!(0, ipv6.flow_label());
            ipv6.set_dscp(10);
            ipv6.set_ecn(3);
            assert_eq!(6, ipv6.version());
            assert_eq!(10, ipv6.dscp());
            assert_eq!(3, ipv6.ecn());
            assert_eq!(0, ipv6.flow_label());
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
