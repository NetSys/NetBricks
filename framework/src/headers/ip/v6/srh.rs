use super::ext::Ipv6ExtHeader;
use super::IpHeader;
use generic_array::typenum::*;
use generic_array::{ArrayLength, GenericArray};
use headers::ip::v6::{Ipv6VarHeader, NextHeader};
use headers::EndOffset;
use std::fmt;
use std::marker::PhantomData;
use std::net::Ipv6Addr;
use std::slice;
use utils::*;

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

// As per Spec: Routing Type: TBD, to be assigned by IANA (suggested value: 4).
pub const ROUTING_TYPE: u8 = 4;

// Positions of the various flag bits in the flag byte
const PROTECTED_FLAG_POS: u8 = 1;
const OAM_FLAG_POS: u8 = 2;
const ALERT_FLAG_POS: u8 = 3;
const HMAC_FLAG_POS: u8 = 4;

pub type Segment = Ipv6Addr;
pub type Segments = [Segment];
pub type SRH<T> = SegmentRoutingHeader<T, U_>;

// Generic Type for parsing/reading possible variable length segment list.
pub type U_ = U0;

// The v6 Segment Routing Header is a specialization of the v6 Routing Header,
// defined in
// https://tools.ietf.org/html/draft-ietf-6man-segment-routing-header-11. Like
// all extension headers, it shares the first two fields.
#[derive(Debug)]
#[repr(C, packed)]
pub struct SegmentRoutingHeader<T, S>
where
    T: Ipv6VarHeader,
    S: ArrayLength<Segment>,
{
    pub ext_header: Ipv6ExtHeader<T>,
    routing_type: u8,
    segments_left: u8,
    last_entry: u8,
    flags: u8,
    tag: u16, // Segments and TLVs follow this, but must be accessed via raw pointers
    segments: GenericArray<Segment, S>,
}

// SRv6 can encapsulate any L4 IP protocol.
impl<T, S> IpHeader for SegmentRoutingHeader<T, S>
where
    T: Ipv6VarHeader,
    S: ArrayLength<Segment>,
{
}

// The SegmentRoutingHeader is an extension header, and so has a next header
// field and can be the PreviousHeader for another header.
impl<T, S> Ipv6VarHeader for SegmentRoutingHeader<T, S>
where
    T: Ipv6VarHeader,
    S: ArrayLength<Segment>,
{
    fn next_header(&self) -> Option<NextHeader> {
        self.ext_header.next_header()
    }
}

// Formats the header for printing
impl<T, S> fmt::Display for SegmentRoutingHeader<T, S>
where
    T: Ipv6VarHeader,
    S: ArrayLength<Segment>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let segments = self.segments().unwrap_or(&[]);
        let num_segments = segments.len();
        let segs = segments
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .join(", ");

        write!(
            f,
            "next_header: {:?}, hdr_ext_len: {}, routing_type: {}, segments_left: {}, last_entry: {}, protected: {}, \
             oam: {}, alert: {},  hmac: {},  tag: {},  segments_length: {},  segments: [{}]",
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
            num_segments,
            segs
        )
    }
}

impl<T, S> EndOffset for SegmentRoutingHeader<T, S>
where
    T: Ipv6VarHeader,
    S: ArrayLength<Segment>,
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

impl<T, S> SegmentRoutingHeader<T, S>
where
    T: Ipv6VarHeader,
    S: ArrayLength<Segment>,
{
    pub fn new(segments: GenericArray<Segment, S>) -> SegmentRoutingHeader<T, S> {
        let num_segments = segments.len() as u8;

        SegmentRoutingHeader {
            ext_header: Ipv6ExtHeader {
                hdr_ext_len: 2 * num_segments,
                ..Default::default()
            },
            routing_type: ROUTING_TYPE,
            segments_left: 0,
            last_entry: num_segments - 1,
            flags: 0,
            tag: 0,
            segments: segments,
        }
    }

    pub fn new_from(
        prev_srh: &SegmentRoutingHeader<T, S>,
        segments: GenericArray<Segment, S>,
    ) -> SegmentRoutingHeader<T, S> {
        let num_segments = segments.len() as u8;

        SegmentRoutingHeader {
            ext_header: Ipv6ExtHeader {
                hdr_ext_len: 2 * num_segments,
                next_header: prev_srh.next_header().unwrap_or(NextHeader::NoNextHeader) as u8,
                _parent: PhantomData,
            },
            routing_type: ROUTING_TYPE,
            segments_left: prev_srh.segments_left(),
            last_entry: num_segments - 1,
            flags: prev_srh.flags(),
            tag: prev_srh.tag(),
            segments: segments,
        }
    }

    pub fn new_from_tuple(
        fields: (Option<NextHeader>, u8, u8, u16),
        segments: GenericArray<Segment, S>,
    ) -> SegmentRoutingHeader<T, S> {
        let num_segments = segments.len() as u8;
        let (next_header, segments_left, flags, tag) = fields;

        SegmentRoutingHeader {
            ext_header: Ipv6ExtHeader {
                hdr_ext_len: 2 * num_segments,
                next_header: next_header.unwrap_or(NextHeader::NoNextHeader) as u8,
                _parent: PhantomData,
            },
            routing_type: ROUTING_TYPE,
            segments_left: segments_left,
            last_entry: num_segments - 1,
            flags: flags,
            tag: tag,
            segments: segments,
        }
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
        get_bit(self.flags, PROTECTED_FLAG_POS)
    }

    pub fn set_protected(&mut self, protected: bool) {
        self.flags = flip_bit(self.flags, PROTECTED_FLAG_POS, protected);
    }

    /// O-flag: OAM flag. When set, it indicates that this packet is an
    /// operations and management (OAM) packet.
    pub fn oam(&self) -> bool {
        get_bit(self.flags, OAM_FLAG_POS)
    }

    pub fn set_oam(&mut self, oam: bool) {
        self.flags = flip_bit(self.flags, OAM_FLAG_POS, oam);
    }

    /// A-flag: Alert flag. If present, it means important Type Length Value
    /// (TLV) objects are present.
    pub fn alert(&self) -> bool {
        get_bit(self.flags, ALERT_FLAG_POS)
    }

    pub fn set_alert(&mut self, alert: bool) {
        self.flags = flip_bit(self.flags, ALERT_FLAG_POS, alert);
    }

    /// H-flag: HMAC flag. If set, the HMAC TLV is present and is encoded as the
    /// last TLV of the SRH. In other words, the last 36 octets of the SRH
    /// represent the HMAC information.
    pub fn hmac(&self) -> bool {
        get_bit(self.flags, HMAC_FLAG_POS)
    }

    pub fn set_hmac(&mut self, hmac: bool) {
        self.flags = flip_bit(self.flags, HMAC_FLAG_POS, hmac);
    }

    pub fn flags(&self) -> u8 {
        u8::from_be(self.flags)
    }

    /// Tag: tag a packet as part of a class or group of packets, e.g., packets
    /// sharing the same set of properties.
    pub fn tag(&self) -> u16 {
        u16::from_be(self.tag)
    }

    pub fn set_tag(&mut self, tag: u16) {
        self.tag = u16::to_be(tag)
    }

    pub fn segments(&self) -> Option<&Segments>
    where
        S: ArrayLength<Segment>,
    {
        let last_entry = self.last_entry();
        let hdr_ext_len = self.hdr_ext_len();

        // o (last_entry + 1) -> number of segments
        // o hdr_ext_len is equal to 2 * number of segments
        // o double check they are both not 0 ahead of time b/c defaults.
        if hdr_ext_len != 0 && (2 * (last_entry + 1) == hdr_ext_len) {
            let num_segments = last_entry as usize + 1;
            let ptr = (self as *const Self) as *const u8;
            unsafe {
                Some(slice::from_raw_parts(
                    ptr.offset(8) as *const Segment,
                    num_segments,
                ))
            }
        } else {
            None
        }
    }
}
