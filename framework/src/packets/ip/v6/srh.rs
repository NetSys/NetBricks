use common::Result;
use failure::Fail;
use native::zcsi::MBuf;
use std::fmt;
use std::net::Ipv6Addr;
use packets::{buffer, Fixed, Header, Packet, ParseError};
use packets::ip::{IpPacket, ProtocolNumber};
use packets::ip::v6::Ipv6Packet;

/*  From https://tools.ietf.org/html/draft-ietf-6man-segment-routing-header-16#section-2
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

    Next Header: 8-bit selector. Identifies the type of header
    immediately following the SRH.

    Hdr Ext Len: 8-bit unsigned integer, is the length of the SRH
    header in 8-octet units, not including the first 8 octets.

    Routing Type: TBD, to be assigned by IANA (suggested value: 4).

    Segments Left: 8-bit unsigned integer Number of route segments 
    remaining, i.e., number of explicitly listed intermediate nodes 
    still to be visited before reaching the final destination.

    Last Entry: contains the index (zero based), in the Segment List,
    of the last element of the Segment List.

    Flags: 8 bits of flags.  Following flags are defined:

         0 1 2 3 4 5 6 7
        +-+-+-+-+-+-+-+-+
        |U U U U U U U U|
        +-+-+-+-+-+-+-+-+

        U: Unused and for future use.  MUST be 0 on transmission and
        ignored on receipt.

    Tag: tag a packet as part of a class or group of packets, e.g.,
    packets sharing the same set of properties. When tag is not used
    at source it MUST be set to zero on transmission. When tag is not
    used during SRH Processing it SHOULD be ignored. The allocation
    and use of tag is outside the scope of this document.

    Segment List[n]: 128 bit IPv6 addresses representing the nth
    segment in the Segment List.  The Segment List is encoded starting
    from the last segment of the SR Policy.  I.e., the first element
    of the segment list (Segment List [0]) contains the last segment
    of the SR Policy, the second element contains the penultimate
    segment of the SR Policy and so on.

    Type Length Value (TLV) are described in Section 2.1.
*/

/// IPv6 segment routing header
/// 
/// The segment routing header contains only the fixed portion of the
/// header. `segment_list` and `tlv` are parsed separately.
#[derive(Debug)]
#[repr(C, packed)]
pub struct SegmentRoutingHeader {
    next_header: u8,
    hdr_ext_len: u8,
    routing_type: u8,
    segments_left: u8,
    last_entry: u8,
    flags: u8,
    tag: u16
}

impl Header for SegmentRoutingHeader {}

/// Type alias for a segment in segment routing header
pub type Segment = Ipv6Addr;

/// Error for a bad segment list
#[derive(Debug, Fail)]
#[fail(display = "Segment list length must be greater than 0")]
pub struct BadSegmentsError(());

pub struct SegmentRouting<E: Ipv6Packet> {
    envelope: E,
    mbuf: *mut MBuf,
    offset: usize,
    header: *mut SegmentRoutingHeader,
    segments: *mut [Segment]
}

impl<E: Ipv6Packet> SegmentRouting<E> {
    #[inline]
    pub fn next_header(&self) -> ProtocolNumber {
        ProtocolNumber::new(self.header().next_header)
    }

    #[inline]
    pub fn set_next_header(&self, next_header: ProtocolNumber) {
        self.header().next_header = next_header.0;
    }

    #[inline]
    pub fn hdr_ext_len(&self) -> u8 {
        self.header().hdr_ext_len
    }

    // internally used by `set_segments`
    #[inline]
    fn set_hdr_ext_len(&self, hdr_ext_len: u8) {
        self.header().hdr_ext_len = hdr_ext_len;
    }

    #[inline]
    pub fn routing_type(&self) -> u8 {
        self.header().routing_type
    }

    #[inline]
    pub fn segments_left(&self) -> u8 {
        self.header().segments_left
    }

    #[inline]
    pub fn set_segments_left(&self, segments_left: u8) {
        self.header().segments_left = segments_left;
    }

    #[inline]
    pub fn last_entry(&self) -> u8 {
        self.header().last_entry
    }

    // internally used by `set_segments`
    #[inline]
    fn set_last_entry(&self, last_entry: u8) {
        self.header().last_entry = last_entry;
    }

    #[inline]
    pub fn flags(&self) -> u8 {
        self.header().flags
    }

    #[inline]
    pub fn tag(&self) -> u16 {
        u16::from_be(self.header().tag)
    }

    #[inline]
    pub fn set_tag(&self, tag: u16) {
        self.header().tag = u16::to_be(tag)
    }

    #[inline]
    pub fn segments(&self) -> &mut [Segment] {
        unsafe { &mut (*self.segments) }
    }

    #[inline]
    pub fn set_segments(&mut self, segments: &[Segment]) -> Result<()> {
        if segments.len() > 0 {
            let old_len = self.last_entry() + 1;
            let new_len = segments.len() as u8;
            let segments_offset = self.offset + SegmentRoutingHeader::size();

            buffer::realloc(self.mbuf, segments_offset, new_len as isize - old_len as isize)?;
            buffer::write_slice(self.mbuf, segments_offset, segments)?;
            self.set_hdr_ext_len(new_len * 2);
            self.set_last_entry(new_len - 1);
            self.segments = buffer::read_slice::<Segment>(self.mbuf, segments_offset, new_len as usize)?;
            Ok(())
        } else {
            Err(BadSegmentsError(()).into())
        }
    }
}

impl<E: Ipv6Packet> fmt::Display for SegmentRouting<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let segments = self.segments()
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .join(", ");

        write!(
            f,
            "next_header: {}, hdr_ext_len: {}, routing_type: {}, segments_left: {}, last_entry: {}, flags: {}, tag: {}, segments: [{}]",
            self.next_header(),
            self.hdr_ext_len(),
            self.routing_type(),
            self.segments_left(),
            self.last_entry(),
            self.flags(),
            self.tag(),
            segments
        )
    }
}

impl<E: Ipv6Packet> Packet for SegmentRouting<E> {
    type Header = SegmentRoutingHeader;
    type Envelope = E;

    #[inline]
    fn from_packet(envelope: Self::Envelope,
                   mbuf: *mut MBuf,
                   offset: usize,
                   header: *mut Self::Header) -> Result<Self> {
        unsafe {
            let hdr_ext_len = (*header).hdr_ext_len;
            let segments_len = (*header).last_entry + 1;

            if hdr_ext_len != 0 && (2 * segments_len == hdr_ext_len) {
                let segments = buffer::read_slice::<Segment>(
                    mbuf,
                    offset + SegmentRoutingHeader::size(),
                    segments_len as usize
                )?;

                Ok(SegmentRouting {
                    envelope,
                    mbuf,
                    offset,
                    header,
                    segments
                })
            } else {
                Err(ParseError::new("Packet has inconsistent segment list length").into())
            }
        }
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
        Self::Header::size() + self.segments().len() * 16
    }
}

impl<E: Ipv6Packet> IpPacket for SegmentRouting<E> {
    fn next_proto(&self) -> ProtocolNumber {
        self.next_header()
    }
}

impl<E: Ipv6Packet> Ipv6Packet for SegmentRouting<E> {}

#[cfg(test)]
mod tests {
    use super::*;
    use packets::{RawPacket, Ethernet};
    use packets::ip::ProtocolNumbers;
    use packets::ip::v6::Ipv6;
    use dpdk_test;

    #[rustfmt::skip]
    pub const SRH_PACKET: [u8; 170] = [
        // ** ethernet header
        0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
        0x86, 0xDD,
        // ** IPv6 Header
        0x60, 0x00, 0x00, 0x00,
        // payload length
        0x00, 0x74,
        // next header (routing)
        0x2b,
        0x02,
        0x20, 0x01, 0x0d, 0xb8, 0x85, 0xa3, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        0x20, 0x01, 0x0d, 0xb8, 0x85, 0xa3, 0x00, 0x00, 0x00, 0x00, 0x8a, 0x2e, 0x03, 0x70, 0x73, 0x34,
        // ** SRv6 header
        // next header (tcp)
        0x06,
        // hdr ext len (3 segments, units of 8 octets)
        0x06,
        // routing type
        0x04,
        // segments left
        0x00,
        // last entry
        0x02,
        // flags
        0x00,
        // tag
        0x00, 0x00,
        // segments[0] 2001:0db8:85a3:0000:0000:8a2e:0370:7333
        0x20, 0x01, 0x0d, 0xb8, 0x85, 0xa3, 0x00, 0x00, 0x00, 0x00, 0x8a, 0x2e, 0x03, 0x70, 0x73, 0x33,
        // segments[1] 2001:0db8:85a3:0000:0000:8a2e:0370:7334
        0x20, 0x01, 0x0d, 0xb8, 0x85, 0xa3, 0x00, 0x00, 0x00, 0x00, 0x8a, 0x2e, 0x03, 0x70, 0x73, 0x34,
        // segments[2] 2001:0db8:85a3:0000:0000:8a2e:0370:7335
        0x20, 0x01, 0x0d, 0xb8, 0x85, 0xa3, 0x00, 0x00, 0x00, 0x00, 0x8a, 0x2e, 0x03, 0x70, 0x73, 0x35,
        // ** TCP header
        // src port
        0x0d, 0x88,
        // dst port
        0x04, 0x00,
        // sequence number
        0x00, 0x00, 0x00, 0x00,
        // ack number
        0x00, 0x00, 0x00, 0x00,
        // flags
        0x50, 0x02,
        // window
        0x00, 0x0a,
        // checksum
        0x00, 0x00,
        // urgent pointer
        0x00, 0x00,
        // payload
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07,
    ];

    #[test]
    fn size_of_segment_routing_header() {
        assert_eq!(8, SegmentRoutingHeader::size());
    }

    #[test]
    fn parse_segment_routing_packet() {
        dpdk_test! {
            let packet = RawPacket::from_bytes(&SRH_PACKET).unwrap();
            let ethernet = packet.parse::<Ethernet>().unwrap();
            let ipv6 = ethernet.parse::<Ipv6>().unwrap();
            let srh = ipv6.parse::<SegmentRouting<Ipv6>>().unwrap();

            assert_eq!(ProtocolNumbers::Tcp, srh.next_header());
            assert_eq!(6, srh.hdr_ext_len());
            assert_eq!(4, srh.routing_type());
            assert_eq!(0, srh.segments_left());
            assert_eq!(2, srh.last_entry());
            assert_eq!(0, srh.flags());
            assert_eq!(0, srh.tag());
            
            let segments = srh.segments();
            assert_eq!(3, segments.len());
            assert_eq!("2001:db8:85a3::8a2e:370:7333", segments[0].to_string());
            assert_eq!("2001:db8:85a3::8a2e:370:7334", segments[1].to_string());
            assert_eq!("2001:db8:85a3::8a2e:370:7335", segments[2].to_string());
        }
    }

    #[test]
    fn set_segments() {
        dpdk_test! {
            let packet = RawPacket::from_bytes(&SRH_PACKET).unwrap();
            let ethernet = packet.parse::<Ethernet>().unwrap();
            let ipv6 = ethernet.parse::<Ipv6>().unwrap();
            let mut srh = ipv6.parse::<SegmentRouting<Ipv6>>().unwrap();

            let segment1: Segment = "::1".parse().unwrap();

            assert!(srh.set_segments(&vec![segment1]).is_ok());
            assert_eq!(2, srh.hdr_ext_len());
            assert_eq!(0, srh.last_entry());
            assert_eq!(1, srh.segments().len());
            assert_eq!(segment1, srh.segments()[0]);

            let segment2: Segment = "::2".parse().unwrap();
            let segment3: Segment = "::3".parse().unwrap();
            let segment4: Segment = "::4".parse().unwrap();

            assert!(srh.set_segments(&vec![segment1, segment2, segment3, segment4]).is_ok());
            assert_eq!(8, srh.hdr_ext_len());
            assert_eq!(3, srh.last_entry());
            assert_eq!(4, srh.segments().len());
            assert_eq!(segment1, srh.segments()[0]);
            assert_eq!(segment2, srh.segments()[1]);
            assert_eq!(segment3, srh.segments()[2]);
            assert_eq!(segment4, srh.segments()[3]);

            assert!(srh.set_segments(&vec![]).is_err());
        }
    }
}
