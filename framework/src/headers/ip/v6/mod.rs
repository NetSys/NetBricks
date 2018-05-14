pub use self::ext::*;
pub use self::srh::*;
use super::IpHeader;
use byteorder::{BigEndian, ByteOrder};
use headers::{EndOffset, MacHeader, TCP_NXT_HDR, UDP_NXT_HDR};
use num::FromPrimitive;
use std::convert::From;
use std::default::Default;
use std::fmt;
use std::net::Ipv6Addr;
use std::slice;
use utils::FlowV6;

mod ext;
mod srh;

/* (From RFC8200 https://tools.ietf.org/html/rfc8200#section-3)
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

   Version              4-bit Internet Protocol version number = 6.

   Traffic Class        8-bit traffic class field.

   Flow Label           20-bit flow label.

   Payload Length       16-bit unsigned integer.  Length of the IPv6
                        payload, i.e., the rest of the packet following
                        this IPv6 header, in octets.  (Note that any
                        extension headers present are considered part of
                        the payload, i.e., included in the length count.)

   Next Header          8-bit selector.  Identifies the type of header
                        immediately following the IPv6 header.  Uses the
                        same values as the IPv4 Protocol field [RFC-1700
                        et seq.].

   Hop Limit            8-bit unsigned integer.  Decremented by 1 by
                        each node that forwards the packet. The packet
                        is discarded if Hop Limit is decremented to
                        zero.

   Source Address       128-bit address of the originator of the packet.

   Destination Address  128-bit address of the intended recipient of the
                        packet (possibly not the ultimate recipient, if
                        a Routing header is present).
*/

// L3 Extention Header Values
const ROUTING_NXT_HDR: u8 = 43;
const HIP_NXT_HDR: u8 = 139;
const MOBILITY_NXT_HDR: u8 = 135;
// TODO: ... more constants here

#[derive(FromPrimitive, Debug, PartialEq)]
#[repr(u8)]
pub enum NextHeader {
    Routing = ROUTING_NXT_HDR,
    HostIdentityProtocol = HIP_NXT_HDR,
    Mobility = MOBILITY_NXT_HDR,
    Tcp = TCP_NXT_HDR,
    Udp = UDP_NXT_HDR,
    NoNextHeader = 59,
}

// V6 addresses are 128 bits wide.
pub type Rawv6Address = u128;

#[derive(Default)]
#[repr(C, packed)]
pub struct Ipv6Header {
    version_to_flow_label: u32,
    payload_len: u16,
    next_header: u8,
    hop_limit: u8,
    src_ip: Rawv6Address,
    dst_ip: Rawv6Address,
}

// IPv6 can encapsulate any L4 IP protocol.
impl IpHeader for Ipv6Header {}

impl EndOffset for Ipv6Header {
    // Note this does not allow IPv6-in-IPv6 or any other tunneling variants.
    type PreviousHeader = MacHeader;

    #[inline]
    fn offset(&self) -> usize {
        // IPv6 Header is always 40 bytes: (4 + 8 + 20 + 16 + 8 + 8 + 128 + 128) / 8 = 40
        40
    }

    #[inline]
    fn size() -> usize {
        // Struct is always 40 bytes as well
        40
    }

    #[inline]
    fn payload_size(&self, _hint: usize) -> usize {
        self.payload_len() as usize
    }

    #[inline]
    fn check_correct(&self, _prev: &MacHeader) -> bool {
        // prev.etype() == 0x86DD
        true
    }
}

// IPv6 Extension headers (and the first header) all have the "next header"
// field which specifies the contents of the following extension or protocol
// header. The value of this field is defined by IANA:
// https://www.iana.org/assignments/protocol-numbers/protocol-numbers.xhtml
//
// Similarly to being generic over v4 and v6 headers as parents for TCP/UDP, we
// make extension headers generic but specify that they need to expose an
// accessor for the next header field.
pub trait Ipv6VarHeader: EndOffset + Default {
    fn next_header(&self) -> Option<NextHeader>;
}

// The IPv6 protocol header has a next header field and can be the
// `PreviousHeader` type for any extension header.
impl Ipv6VarHeader for Ipv6Header {
    fn next_header(&self) -> Option<NextHeader> {
        self.next_header()
    }
}

// Formats the header for printing
impl fmt::Display for Ipv6Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let src = Ipv6Addr::from(self.src());
        let dst = Ipv6Addr::from(self.dst());
        write!(
            f,
            "{} > {} version: {} traffic_class: {} flow_label: {} len: {} next_header: {:?} hop_limit: {}",
            src,
            dst,
            self.version(),
            self.traffic_class(),
            self.flow_label(),
            self.payload_len(),
            self.next_header().unwrap(),
            self.hop_limit()
        )
    }
}

impl Ipv6Header {
    #[inline]
    pub fn flow(&self) -> Option<FlowV6> {
        if let Some(next_hdr) = self.next_header() {
            let src_ip = self.src();
            let dst_ip = self.dst();
            if (next_hdr == NextHeader::Tcp || next_hdr == NextHeader::Udp)
                && self.payload_size(0) >= 4
            {
                unsafe {
                    let self_as_u8 = (self as *const Ipv6Header) as *const u8;
                    let port_as_u8 = self_as_u8.offset(self.offset() as isize);
                    let port_slice = slice::from_raw_parts(port_as_u8, 4);
                    let dst_port = BigEndian::read_u16(&port_slice[..2]);
                    let src_port = BigEndian::read_u16(&port_slice[2..]);
                    Some(FlowV6 {
                        src_ip: src_ip,
                        dst_ip: dst_ip,
                        src_port: src_port,
                        dst_port: dst_port,
                        proto: next_hdr as u8,
                    })
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    #[inline]
    pub fn new() -> Ipv6Header {
        Default::default()
    }

    // Source address (converted to host byte order)
    #[inline]
    pub fn src(&self) -> Rawv6Address {
        Rawv6Address::from_be(self.src_ip)
    }

    #[inline]
    pub fn set_src(&mut self, src: Rawv6Address) {
        self.src_ip = Rawv6Address::to_be(src)
    }

    // Destination address (converted to host byte order)
    #[inline]
    pub fn dst(&self) -> Rawv6Address {
        Rawv6Address::from_be(self.dst_ip)
    }

    #[inline]
    pub fn set_dst(&mut self, dst: Rawv6Address) {
        self.dst_ip = Rawv6Address::to_be(dst);
    }

    // Hop Limit (TTL)
    #[inline]
    pub fn hop_limit(&self) -> u8 {
        self.hop_limit
    }

    #[inline]
    pub fn set_hop_limit(&mut self, hlimit: u8) {
        self.hop_limit = hlimit;
    }

    // Protocol Version, should always be `6`
    #[inline]
    pub fn version(&self) -> u8 {
        ((u32::from_be(self.version_to_flow_label) & 0xf0000000) >> 28) as u8
    }

    #[inline]
    pub fn set_version(&mut self, version: u8) {
        self.version_to_flow_label = u32::to_be(
            (((version as u32) << 28) & 0xf0000000)
                | (u32::from_be(self.version_to_flow_label) & !0xf0000000),
        );
    }

    // Traffic class field
    #[inline]
    pub fn traffic_class(&self) -> u8 {
        ((u32::from_be(self.version_to_flow_label) >> 20) as u8)
    }

    #[inline]
    pub fn set_traffic_class(&mut self, tclass: u8) {
        self.version_to_flow_label = u32::to_be(
            (u32::from_be(self.version_to_flow_label) & 0xf00fffff) | ((tclass as u32) << 20),
        )
    }

    // Flow label field
    #[inline]
    pub fn flow_label(&self) -> u32 {
        u32::from_be(self.version_to_flow_label) & 0x0fffff
    }

    #[inline]
    pub fn set_flow_label(&mut self, flow_label: u32) {
        assert!(flow_label <= 0x0fffff);
        self.version_to_flow_label = u32::to_be(
            (u32::from_be(self.version_to_flow_label) & 0xfff00000) | (flow_label & 0x0fffff),
        )
    }

    // Size of the contained payload, including any extension headers
    #[inline]
    pub fn payload_len(&self) -> u16 {
        u16::from_be(self.payload_len)
    }

    #[inline]
    pub fn set_payload_len(&mut self, len: u16) {
        self.payload_len = u16::to_be(len)
    }

    // The number of the next protocol header in the payload, either an
    // extension header or an L4 protocol.
    #[inline]
    pub fn next_header(&self) -> Option<NextHeader> {
        FromPrimitive::from_u8(self.next_header)
    }

    #[inline]
    pub fn set_next_header(&mut self, hdr: NextHeader) {
        self.next_header = hdr as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv6Addr;
    use std::str::FromStr;

    #[test]
    fn packet() {
        let mut ip = Ipv6Header::new();
        let src = Ipv6Addr::from_str("2001:db8::1").unwrap();
        let dst = Ipv6Addr::from_str("2001:db8::2").unwrap();
        ip.set_src(u128::from(src));
        ip.set_dst(u128::from(dst));
        ip.set_version(6);
        ip.set_traffic_class(17);
        ip.set_flow_label(15000);
        ip.set_payload_len(1000);
        ip.set_next_header(NextHeader::Udp);
        ip.set_hop_limit(2);

        assert_eq!(ip.version(), 6);
        assert_eq!(ip.traffic_class(), 17);
        assert_eq!(ip.flow_label(), 15000);
        assert_eq!(ip.payload_len(), 1000);
        assert_eq!(ip.next_header().unwrap(), NextHeader::Udp);
        assert_eq!(ip.next_header, UDP_NXT_HDR);
        assert_eq!(ip.hop_limit(), 2);
        assert_eq!("2001:db8::1 > 2001:db8::2 version: 6 traffic_class: 17 flow_label: 15000 len: 1000 next_header: Udp hop_limit: 2", ip.to_string())
    }
}
