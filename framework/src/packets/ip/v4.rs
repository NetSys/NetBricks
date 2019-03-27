use common::Result;
use native::zcsi::MBuf;
use packets::ip::{IpAddrMismatchError, IpPacket, ProtocolNumber};
use packets::{buffer, Ethernet, Fixed, Header, Packet};
use std::fmt;
use std::net::{IpAddr, Ipv4Addr};
use std::slice;

/*  From (https://tools.ietf.org/html/rfc791#section-3.1)
    Internet Datagram Header

     0                   1                   2                   3
     0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |Version|  IHL  |Type of Service|          Total Length         |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |         Identification        |Flags|      Fragment Offset    |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |  Time to Live |    Protocol   |         Header Checksum       |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |                       Source Address                          |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |                    Destination Address                        |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |                    Options                    |    Padding    |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

    Version:  4 bits
        The Version field indicates the format of the internet header.  This
        document describes version 4.

    IHL:  4 bits
        Internet Header Length is the length of the internet header in 32
        bit words, and thus points to the beginning of the data.  Note that
        the minimum value for a correct header is 5.

    Type of Service:  8 bits
        The Type of Service provides an indication of the abstract
        parameters of the quality of service desired.

    Total Length:  16 bits
        Total Length is the length of the datagram, measured in octets,
        including internet header and data.

    Identification:  16 bits
        An identifying value assigned by the sender to aid in assembling the
        fragments of a datagram.

    Flags:  3 bits
        Various Control Flags.

        Bit 0: reserved, must be zero
        Bit 1: (DF) 0 = May Fragment,  1 = Don't Fragment.
        Bit 2: (MF) 0 = Last Fragment, 1 = More Fragments.

          0   1   2
        +---+---+---+
        |   | D | M |
        | 0 | F | F |
        +---+---+---+

    Fragment Offset:  13 bits
        This field indicates where in the datagram this fragment belongs.
        The fragment offset is measured in units of 8 octets (64 bits).  The
        first fragment has offset zero.

    Time to Live:  8 bits
        This field indicates the maximum time the datagram is allowed to
        remain in the internet system.  If this field contains the value
        zero, then the datagram must be destroyed.  This field is modified
        in internet header processing.  The time is measured in units of
        seconds, but since every module that processes a datagram must
        decrease the TTL by at least one even if it process the datagram in
        less than a second, the TTL must be thought of only as an upper
        bound on the time a datagram may exist.  The intention is to cause
        undeliverable datagrams to be discarded, and to bound the maximum
        datagram lifetime.

    Protocol:  8 bits
        This field indicates the next level protocol used in the data
        portion of the internet datagram.  The values for various protocols
        are specified in "Assigned Numbers".

    Header Checksum:  16 bits
        A checksum on the header only.  Since some header fields change
        (e.g., time to live), this is recomputed and verified at each point
        that the internet header is processed.

    Source Address:  32 bits
        The source address.

    Destination Address:  32 bits
        The destination address.

    Options:  variable
        The options may appear or not in datagrams.  They must be
        implemented by all IP modules (host and gateways).  What is optional
        is their transmission in any particular datagram, not their
        implementation.
*/

/// IPv4 header
///
/// The header only include the fixed portion of the IPv4 header.
/// Options are parsed separately.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct Ipv4Header {
    version_ihl: u8,
    type_of_service: u8,
    total_length: u16,
    identification: u16,
    flags_to_frag_offset: u16,
    ttl: u8,
    protocol: u8,
    checksum: u16,
    src: Ipv4Addr,
    dst: Ipv4Addr,
}

impl Default for Ipv4Header {
    fn default() -> Ipv4Header {
        Ipv4Header {
            version_ihl: 4 << 4,
            type_of_service: 0,
            total_length: 0,
            identification: 0,
            flags_to_frag_offset: 0,
            ttl: 0,
            protocol: 0,
            checksum: 0,
            src: Ipv4Addr::UNSPECIFIED,
            dst: Ipv4Addr::UNSPECIFIED,
        }
    }
}

impl Header for Ipv4Header {}

/// IPv4 packet
pub struct Ipv4 {
    envelope: Ethernet,
    mbuf: *mut MBuf,
    offset: usize,
    header: *mut Ipv4Header,
}

impl Ipv4 {
    #[inline]
    pub fn version(&self) -> u8 {
        // Protocol Version, should always be `4`
        (self.header().version_ihl & 0xf0) >> 4
    }

    #[inline]
    pub fn ihl(&self) -> u8 {
        self.header().version_ihl & 0x0f
    }

    #[inline]
    pub fn set_ihl(&self, ihl: u8) {
        self.header().version_ihl = (self.header().version_ihl & 0x0f) | (ihl & 0x0f);
    }

    #[inline]
    pub fn type_of_service(&self) -> u8 {
        self.header().type_of_service
    }

    #[inline]
    pub fn set_type_of_service(&self, type_of_service: u8) {
        self.header().type_of_service = type_of_service;
    }

    #[inline]
    pub fn total_length(&self) -> u16 {
        u16::from_be(self.header().total_length)
    }

    #[inline]
    pub fn set_total_length(&self, total_length: u16) {
        self.header().total_length = u16::to_be(total_length);
    }

    #[inline]
    pub fn identification(&self) -> u16 {
        u16::from_be(self.header().identification)
    }

    #[inline]
    pub fn set_identification(&self, identification: u16) {
        self.header().identification = u16::to_be(identification);
    }

    #[inline]
    pub fn ttl(&self) -> u8 {
        self.header().ttl
    }

    #[inline]
    pub fn set_ttl(&self, ttl: u8) {
        self.header().ttl = ttl;
    }

    #[inline]
    pub fn protocol(&self) -> ProtocolNumber {
        ProtocolNumber::new(self.header().protocol)
    }

    #[inline]
    pub fn set_protocol(&self, protocol: ProtocolNumber) {
        self.header().protocol = protocol.0;
    }

    #[inline]
    pub fn checksum(&self) -> u16 {
        u16::from_be(self.header().checksum)
    }

    #[inline]
    pub fn set_checksum(&self, checksum: u16) {
        self.header().checksum = u16::to_be(checksum);
    }

    #[inline]
    pub fn src(&self) -> Ipv4Addr {
        self.header().src
    }

    #[inline]
    fn set_src(&self, src: Ipv4Addr) {
        self.header().src = src;
    }

    #[inline]
    pub fn dst(&self) -> Ipv4Addr {
        self.header().dst
    }

    #[inline]
    fn set_dst(&self, dst: Ipv4Addr) {
        self.header().dst = dst;
    }
}

impl fmt::Display for Ipv4 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} > {} version: {}, ihl: {}, len: {}, ttl: {}, protocol: {}, checksum: {}",
            self.src(),
            self.dst(),
            self.version(),
            self.ihl(),
            self.total_length(),
            self.ttl(),
            self.protocol(),
            self.checksum()
        )
    }
}

impl Packet for Ipv4 {
    type Header = Ipv4Header;
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

        Ok(Ipv4 {
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

        Ok(Ipv4 {
            envelope,
            mbuf,
            offset,
            header,
        })
    }
}

impl IpPacket for Ipv4 {
    #[inline]
    fn next_proto(&self) -> ProtocolNumber {
        self.protocol()
    }

    #[inline]
    fn src(&self) -> IpAddr {
        IpAddr::V4(self.src())
    }

    #[inline]
    fn set_src(&self, src: IpAddr) -> Result<()> {
        match src {
            IpAddr::V4(addr) => {
                self.set_src(addr);
                Ok(())
            }
            _ => Err(IpAddrMismatchError.into()),
        }
    }

    #[inline]
    fn dst(&self) -> IpAddr {
        IpAddr::V4(self.dst())
    }

    #[inline]
    fn set_dst(&self, dst: IpAddr) -> Result<()> {
        match dst {
            IpAddr::V4(addr) => {
                self.set_dst(addr);
                Ok(())
            }
            _ => Err(IpAddrMismatchError.into()),
        }
    }

    /// Returns the IPv4 pseudo-header sum
    ///
    ///  0      7 8     15 16    23 24    31
    /// +--------+--------+--------+--------+
    /// |          source address           |
    /// +--------+--------+--------+--------+
    /// |        destination address        |
    /// +--------+--------+--------+--------+
    /// |  zero  |protocol|  packet length  |
    /// +--------+--------+--------+--------+
    #[inline]
    fn pseudo_header_sum(&self, packet_len: u16, protocol: ProtocolNumber) -> u16 {
        // a bit of unsafe magic to cast [u8; 4] to [u16; 2]
        let src =
            unsafe { slice::from_raw_parts((&self.src().octets()).as_ptr() as *const u16, 2) };
        let dst =
            unsafe { slice::from_raw_parts((&self.dst().octets()).as_ptr() as *const u16, 2) };

        let mut sum = src[0] as u32
            + src[1] as u32
            + dst[0] as u32
            + dst[1] as u32
            + protocol.0 as u32
            + packet_len as u32;

        while sum >> 16 != 0 {
            sum = (sum >> 16) + (sum & 0xFFFF);
        }

        sum as u16
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dpdk_test;
    use packets::ip::ProtocolNumbers;
    use packets::{Ethernet, RawPacket};

    #[test]
    fn size_of_ipv4_header() {
        assert_eq!(20, Ipv4Header::size());
    }

    #[test]
    fn parse_ipv4_packet() {
        use packets::udp::tests::UDP_PACKET;

        dpdk_test! {
            let packet = RawPacket::from_bytes(&UDP_PACKET).unwrap();
            let ethernet = packet.parse::<Ethernet>().unwrap();
            let ipv4 = ethernet.parse::<Ipv4>().unwrap();

            assert_eq!(4, ipv4.version());
            assert_eq!(5, ipv4.ihl());
            assert_eq!(38, ipv4.total_length());
            assert_eq!(43849, ipv4.identification());
            assert_eq!(255, ipv4.ttl());
            assert_eq!(ProtocolNumbers::Udp, ipv4.protocol());
            assert_eq!(0xf700, ipv4.checksum());
            assert_eq!("139.133.217.110", ipv4.src().to_string());
            assert_eq!("139.133.233.2", ipv4.dst().to_string());
        }
    }

    #[test]
    fn push_ipv4_packet() {
        dpdk_test! {
            let packet = RawPacket::new().unwrap();
            let ethernet = packet.push::<Ethernet>().unwrap();
            let ipv4 = ethernet.push::<Ipv4>().unwrap();

            assert_eq!(4, ipv4.version());
            assert_eq!(Ipv4Header::size(), ipv4.len());
        }
    }
}
