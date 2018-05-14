use super::ext::Ipv6ExtHeader;
use super::IpHeader;
use headers::ip::v6::{Ipv6VarHeader, NextHeader};
use headers::EndOffset;
use std::default::Default;
use std::fmt;
use std::slice;

/* From the SRH Draft RFC
   https://tools.ietf.org/html/draft-ietf-6man-segment-routing-header-11#section-3

   Segment Routing Extension Header (SRH)

   A new type of the Routing Header (originally defined in [RFC8200]) is
   defined: the Segment Routing Header (SRH) which has a new Routing
   Type, (suggested value 4) to be assigned by IANA.

   The Segment Routing Header (SRH) is defined as follows:

     0                   1                   2                   3
     0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    | Next Header   |  Hdr Ext Len  | Routing Type  | Segments Left |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |  Last Entry   |     Flags     |              Tag              |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |                                                               |
    |            Segment List[0] (128 bits IPv6 address)            |
    |                                                               |
    |                                                               |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |                                                               |
    |                                                               |
                                  ...
    |                                                               |
    |                                                               |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |                                                               |
    |            Segment List[n] (128 bits IPv6 address)            |
    |                                                               |
    |                                                               |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    //                                                             //
    //         Optional Type Length Value objects (variable)       //
    //                                                             //
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

   where:

   o  Next Header: 8-bit selector.  Identifies the type of header
      immediately following the SRH.

   o  Hdr Ext Len: 8-bit unsigned integer, is the length of the SRH
      header in 8-octet units, not including the first 8 octets.

   o  Routing Type: TBD, to be assigned by IANA (suggested value: 4).

   o  Segments Left.  Defined in [RFC8200], it contains the index, in
      the Segment List, of the next segment to inspect.  Segments Left
      is decremented at each segment.

   o  Last Entry: contains the index, in the Segment List, of the last
      element of the Segment List.

   o  Flags: 8 bits of flags.  Following flags are defined:

          0 1 2 3 4 5 6 7
         +-+-+-+-+-+-+-+-+
         |U|P|O|A|H|  U  |
         +-+-+-+-+-+-+-+-+

         U: Unused and for future use.  SHOULD be unset on transmission
         and MUST be ignored on receipt.

         P-flag: Protected flag.  Set when the packet has been rerouted
         through FRR mechanism by an SR endpoint node.

         O-flag: OAM flag.  When set, it indicates that this packet is
         an operations and management (OAM) packet.

         A-flag: Alert flag.  If present, it means important Type Length
         Value (TLV) objects are present.  See Section 3.1 for details
         on TLVs objects.

         H-flag: HMAC flag.  If set, the HMAC TLV is present and is
         encoded as the last TLV of the SRH.  In other words, the last
         36 octets of the SRH represent the HMAC information.  See
         Section 3.1.5 for details on the HMAC TLV.

   o  Tag: tag a packet as part of a class or group of packets, e.g.,
      packets sharing the same set of properties.

   o  Segment List[n]: 128 bit IPv6 addresses representing the nth
      segment in the Segment List.  The Segment List is encoded starting
      from the last segment of the path.  I.e., the first element of the
      segment list (Segment List [0]) contains the last segment of the
      path, the second element contains the penultimate segment of the
      path and so on.

   o  Type Length Value (TLV) are described in Section 3.1.
*/

// SRv6 Segment IDs are an array of 128-bit values with a length defined at
// runtime.
pub type Segments = [u128];

// The v6 Segment Routing Header is a specialization of the v6 Routing Header,
// defined in
// https://tools.ietf.org/html/draft-ietf-6man-segment-routing-header-11. Like
// all extension headers, it shares the first two fields.
#[derive(Default)]
#[repr(C, packed)]
pub struct SegmentRoutingHeader<T>
where
    T: Ipv6VarHeader,
{
    ext_header: Ipv6ExtHeader<T>,
    routing_type: u8,
    segments_left: u8,
    last_entry: u8,
    flags: u8,
    tag: u16, // Segments and TLVs follow this, but must be accessed via raw pointers
}

// SRv6 can encapsulate any L4 IP protocol.
impl<T> IpHeader for SegmentRoutingHeader<T>
where
    T: Ipv6VarHeader,
{
}

// The SegmentRoutingHeader is an extension header, and so has a next header
// field and can be the PreviousHeader for another header.
impl<T> Ipv6VarHeader for SegmentRoutingHeader<T>
where
    T: Ipv6VarHeader,
{
    fn next_header(&self) -> Option<NextHeader> {
        self.ext_header.next_header()
    }
}

// Formats the header for printing
impl<T> fmt::Display for SegmentRoutingHeader<T>
where
    T: Ipv6VarHeader,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "next_header: {:?} hdr_ext_len: {} routing_type: {} segments_left: {} last_entry: {} protected: {} oam: {} \
            alert: {} hmac: {} tag: {} segments_length: {}",
            self.next_header().unwrap_or(NextHeader::NoNextHeader),
            self.hdr_ext_len(),
            self.routing_type(),
            self.segments_left(),
            self.last_entry(),
            self.protected(),
            self.oam(),
            self.alert(),
            self.hmac(),
            self.tag(),
            self.segments().len()
        )
    }
}

impl<T> EndOffset for SegmentRoutingHeader<T>
where
    T: Ipv6VarHeader,
{
    type PreviousHeader = T;

    #[inline]
    fn offset(&self) -> usize {
        // The routing header offset is the same as defined by the extension
        // header.
        self.ext_header.offset()
    }

    #[inline]
    fn size() -> usize {
        // SRH size includes only the known/static parts of the header, we do
        // not know until runtime the length of the entire header, including the
        // SIDs and TLVs. However, this function is used to ensure that the
        // payload in the MBuf is at least as large as the struct size.
        (8 as usize)
    }

    #[inline]
    fn payload_size(&self, hint: usize) -> usize {
        // Same as the extension header payload_size, we rely on the parent's
        // suggested payload size to determine the remainder after this header.
        hint - self.offset()
    }

    #[inline]
    fn check_correct(&self, _prev: &Self::PreviousHeader) -> bool {
        // _prev.next_header() == 43 && self.routing_type == 4
        true
    }
}

impl<T> SegmentRoutingHeader<T>
where
    T: Ipv6VarHeader,
{
    pub fn new() -> SegmentRoutingHeader<T> {
        Default::default()
    }

    pub fn hdr_ext_len(&self) -> u8 {
        self.ext_header.hdr_ext_len
    }

    // Routing Type: TBD, to be assigned by IANA (suggested value: 4).
    pub fn routing_type(&self) -> u8 {
        self.routing_type
    }

    pub fn set_routing_type(&mut self, routing_type: u8) {
        self.routing_type = routing_type;
    }

    pub fn segments_left(&self) -> u8 {
        self.segments_left
    }

    pub fn set_segments_left(&mut self, segments_left: u8) {
        self.segments_left = segments_left;
    }

    pub fn last_entry(&self) -> u8 {
        self.last_entry
    }

    pub fn set_last_entry(&mut self, last_entry: u8) {
        self.last_entry = last_entry;
    }

    /// P-flag: Protected flag.  Set when the packet has been rerouted
    /// through FRR mechanism by an SR endpoint node.
    pub fn protected(&self) -> bool {
        (self.flags & 0x40) > 0
    }

    pub fn set_protected(&mut self, protected: bool) {
        let bit: u8 = if protected { 0x40 } else { 0 };
        self.flags = (self.flags & !0x40) | bit;
    }

    /// O-flag: OAM flag. When set, it indicates that this packet is an
    /// operations and management (OAM) packet.
    pub fn oam(&self) -> bool {
        (self.flags & 0x20) > 0
    }

    pub fn set_oam(&mut self, oam: bool) {
        let bit: u8 = if oam { 0x20 } else { 0 };
        self.flags = (self.flags & !0x20) | bit;
    }

    /// A-flag: Alert flag. If present, it means important Type Length Value
    /// (TLV) objects are present.
    pub fn alert(&self) -> bool {
        (self.flags & 0x10) > 0
    }

    pub fn set_alert(&mut self, alert: bool) {
        let bit: u8 = if alert { 0x10 } else { 0 };
        self.flags = (self.flags & !0x10) | bit;
    }

    /// H-flag: HMAC flag. If set, the HMAC TLV is present and is encoded as the
    /// last TLV of the SRH. In other words, the last 36 octets of the SRH
    /// represent the HMAC information.
    pub fn hmac(&self) -> bool {
        (self.flags & 0x08) > 0
    }

    pub fn set_hmac(&mut self, hmac: bool) {
        let bit: u8 = if hmac { 0x08 } else { 0 };
        self.flags = (self.flags & !0x08) | bit;
    }

    /// Tag: tag a packet as part of a class or group of packets, e.g., packets
    /// sharing the same set of properties.
    pub fn tag(&self) -> u16 {
        u16::from_be(self.tag)
    }

    pub fn set_tag(&mut self, tag: u16) {
        self.tag = u16::to_be(tag)
    }

    pub fn segments(&self) -> &Segments {
        // TODO: check that hdr_ext_len and last_entry agree on the number of
        // segments
        let num_segments = self.last_entry() as usize + 1;
        let ptr = (self as *const Self) as *const u8;
        unsafe { slice::from_raw_parts(ptr.offset(8) as *const u128, num_segments) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use headers::tests::packet_from_bytes;
    use headers::{EtherType, Ipv6Header, MacAddress, MacHeader};
    use std::convert::From;
    use std::net::Ipv6Addr;
    use std::str::FromStr;

    #[test]
    fn srh_from_bytes() {
        let packet_header = [
            // --- Ethernet header ---
            // Destination MAC
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x01,
            // Source MAC
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x02,
            // EtherType (IPv6)
            0x86,
            0xDD,
            // --- IPv6 Header ---
            // Version, Traffic Class, Flow Label
            0x60,
            0x00,
            0x00,
            0x00,
            // Payload Length
            0x00,
            0x18,
            // Next Header (Routing = 43)
            0x2b,
            // Hop Limit
            0x02,
            // Source Address
            0x20,
            0x01,
            0x0d,
            0xb8,
            0x85,
            0xa3,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x01,
            // Dest Address
            0x20,
            0x01,
            0x0d,
            0xb8,
            0x85,
            0xa3,
            0x00,
            0x00,
            0x00,
            0x00,
            0x8a,
            0x2e,
            0x03,
            0x70,
            0x73,
            0x34,
            // --- SRv6 Header --
            // Next Header (TCP)
            0x11,
            // Hdr Ext Len (just one segment, units of 8 octets or 64 bits)
            0x04,
            // Routing type (SRv6)
            0x04,
            // Segments left
            0x00,
            // Last entry
            0x01,
            // Flags
            0x00,
            // Tag
            0x00,
            0x00,
            // Segments: [0] 2001:0db8:85a3:0000:0000:8a2e:0370:7334
            0x20,
            0x01,
            0x0d,
            0xb8,
            0x85,
            0xa3,
            0x00,
            0x00,
            0x00,
            0x00,
            0x8a,
            0x2e,
            0x03,
            0x70,
            0x73,
            0x34,
            // Segments: [1] 2001:0db8:85a3:0000:0000:8a2e:0370:7335
            0x20,
            0x01,
            0x0d,
            0xb8,
            0x85,
            0xa3,
            0x00,
            0x00,
            0x00,
            0x00,
            0x8a,
            0x2e,
            0x03,
            0x70,
            0x73,
            0x35,
        ];
        let pkt = packet_from_bytes(&packet_header);

        // Check Ethernet header
        let epkt = pkt.parse_header::<MacHeader>();
        {
            let eth = epkt.get_header();
            assert_eq!(eth.dst.addr, MacAddress::new(0, 0, 0, 0, 0, 1).addr);
            assert_eq!(eth.src.addr, MacAddress::new(0, 0, 0, 0, 0, 2).addr);
            assert_eq!(eth.etype(), Some(EtherType::IPv6));
        }

        // Check IPv6 header
        let v6pkt = epkt.parse_header::<Ipv6Header>();
        {
            let v6 = v6pkt.get_header();
            let src = Ipv6Addr::from_str("2001:db8:85a3::1").unwrap();
            let dst = Ipv6Addr::from_str("2001:db8:85a3::8a2e:0370:7334").unwrap();
            assert_eq!(v6.version(), 6);
            assert_eq!(v6.traffic_class(), 0);
            assert_eq!(v6.flow_label(), 0);
            assert_eq!(v6.payload_len(), 24);
            assert_eq!(v6.next_header().unwrap(), NextHeader::Routing);
            assert_eq!(v6.hop_limit(), 2);
            assert_eq!(Ipv6Addr::from(v6.src()), src);
            assert_eq!(Ipv6Addr::from(v6.dst()), dst);
        }

        // Check SRH
        let srhpkt = v6pkt.parse_header::<SegmentRoutingHeader<Ipv6Header>>();
        {
            let srh = srhpkt.get_header();
            let seg0 = Ipv6Addr::from_str("2001:db8:85a3::8a2e:0370:7334").unwrap();
            let seg1 = Ipv6Addr::from_str("2001:db8:85a3::8a2e:0370:7335").unwrap();
            assert_eq!(srh.next_header().unwrap(), NextHeader::Udp);
            assert_eq!(srh.hdr_ext_len(), 4);
            assert_eq!(srh.routing_type(), 4);
            assert_eq!(srh.segments_left(), 0);
            assert_eq!(srh.last_entry(), 1);
            assert_eq!(srh.protected(), false);
            assert_eq!(srh.oam(), false);
            assert_eq!(srh.alert(), false);
            assert_eq!(srh.hmac(), false);
            assert_eq!(srh.tag(), 0);
            assert_eq!(srh.segments().len(), 2);
            assert_eq!(Ipv6Addr::from(u128::from_be(srh.segments()[0])), seg0);
            assert_eq!(Ipv6Addr::from(u128::from_be(srh.segments()[1])), seg1);
        }
    }
}
