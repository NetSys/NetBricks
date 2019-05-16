use common::Result;
use native::zcsi::MBuf;
use packets::ip::{Flow, IpPacket, ProtocolNumbers};
use packets::{buffer, checksum, Fixed, Header, Packet};
use std::fmt;
use std::net::IpAddr;

/*  From https://tools.ietf.org/html/rfc768
    User Datagram Header Format

     0      7 8     15 16    23 24    31
    +--------+--------+--------+--------+
    |     Source      |   Destination   |
    |      Port       |      Port       |
    +--------+--------+--------+--------+
    |                 |                 |
    |     Length      |    Checksum     |
    +--------+--------+--------+--------+
    |
    |          data octets ...
    +---------------- ...

    Source Port is an optional field, when meaningful, it indicates the port
    of the sending  process,  and may be assumed  to be the port  to which a
    reply should  be addressed  in the absence of any other information.  If
    not used, a value of zero is inserted.

    Destination  Port has a meaning  within  the  context  of  a  particular
    internet destination address.

    Length  is the length  in octets  of this user datagram  including  this
    header  and the data.   (This  means  the minimum value of the length is
    eight.)

    Checksum is the 16-bit one's complement of the one's complement sum of a
    pseudo header of information from the IP header, the UDP header, and the
    data,  padded  with zero octets  at the end (if  necessary)  to  make  a
    multiple of two octets.

    The pseudo  header  conceptually prefixed to the UDP header contains the
    source  address,  the destination  address,  the protocol,  and the  UDP
    length.   This information gives protection against misrouted datagrams.
    This checksum procedure is the same as is used in TCP.

                 0      7 8     15 16    23 24    31
                +--------+--------+--------+--------+
                |          source address           |
                +--------+--------+--------+--------+
                |        destination address        |
                +--------+--------+--------+--------+
                |  zero  |protocol|   UDP length    |
                +--------+--------+--------+--------+

    If the computed  checksum  is zero,  it is transmitted  as all ones (the
    equivalent  in one's complement  arithmetic).   An all zero  transmitted
    checksum  value means that the transmitter  generated  no checksum  (for
    debugging or for higher level protocols that don't care).
*/

/// UDP header
#[derive(Debug, Default, Copy, Clone)]
#[repr(C)]
pub struct UdpHeader {
    src_port: u16,
    dst_port: u16,
    length: u16,
    checksum: u16,
}

impl Header for UdpHeader {}

/// UDP packet
#[derive(Debug)]
pub struct Udp<E: IpPacket> {
    envelope: E,
    mbuf: *mut MBuf,
    offset: usize,
    header: *mut UdpHeader,
}

impl<E: IpPacket> Udp<E> {
    #[inline]
    pub fn src_port(&self) -> u16 {
        u16::from_be(self.header().src_port)
    }

    #[inline]
    pub fn set_src_port(&mut self, src_port: u16) {
        self.header_mut().src_port = u16::to_be(src_port);
    }

    #[inline]
    pub fn dst_port(&self) -> u16 {
        u16::from_be(self.header().dst_port)
    }

    #[inline]
    pub fn set_dst_port(&mut self, dst_port: u16) {
        self.header_mut().dst_port = u16::to_be(dst_port);
    }

    #[inline]
    pub fn length(&self) -> u16 {
        u16::from_be(self.header().length)
    }

    #[inline]
    fn set_length(&mut self, length: u16) {
        self.header_mut().length = u16::to_be(length);
    }

    #[inline]
    pub fn checksum(&self) -> u16 {
        u16::from_be(self.header().checksum)
    }

    #[inline]
    fn set_checksum(&mut self, checksum: u16) {
        // For UDP, if the computed checksum is zero, it is transmitted as
        // all ones. An all zero transmitted checksum value means that the
        // transmitter generated no checksum. To set the checksum value to
        // `0`, use `no_checksum` instead of `set_checksum`.
        self.header_mut().checksum = match checksum {
            0 => 0xFFFF,
            _ => u16::to_be(checksum),
        }
    }

    /// Sets checksum to 0 indicating no checksum generated
    #[inline]
    pub fn no_checksum(&mut self) {
        self.header_mut().checksum = 0;
    }

    #[inline]
    pub fn flow(&self) -> Flow {
        Flow::new(
            self.envelope().src(),
            self.envelope().dst(),
            self.src_port(),
            self.dst_port(),
            ProtocolNumbers::Udp,
        )
    }

    /// Sets the layer-3 source address and recomputes the checksum
    #[inline]
    pub fn set_src_ip(&mut self, src_ip: IpAddr) -> Result<()> {
        let old_ip = self.envelope().src();
        let checksum = checksum::compute_with_ipaddr(self.checksum(), &old_ip, &src_ip)?;
        self.envelope_mut().set_src(src_ip)?;
        self.set_checksum(checksum);
        Ok(())
    }

    /// Sets the layer-3 destination address and recomputes the checksum
    #[inline]
    pub fn set_dst_ip(&mut self, dst_ip: IpAddr) -> Result<()> {
        let old_ip = self.envelope().dst();
        let checksum = checksum::compute_with_ipaddr(self.checksum(), &old_ip, &dst_ip)?;
        self.envelope_mut().set_dst(dst_ip)?;
        self.set_checksum(checksum);
        Ok(())
    }

    #[inline]
    fn compute_checksum(&mut self) {
        self.no_checksum();

        if let Ok(data) = buffer::read_slice(self.mbuf, self.offset, self.len()) {
            let data = unsafe { &(*data) };
            let pseudo_header_sum = self
                .envelope()
                .pseudo_header(data.len() as u16, ProtocolNumbers::Udp)
                .sum();
            let checksum = checksum::compute(pseudo_header_sum, data);
            self.set_checksum(checksum);
        } else {
            // we are reading till the end of buffer, should never run out
            unreachable!()
        }
    }
}

impl<E: IpPacket> fmt::Display for Udp<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "src_port: {}, dst_port: {}, length: {}, checksum: {}",
            self.src_port(),
            self.dst_port(),
            self.length(),
            self.checksum()
        )
    }
}

impl<E: IpPacket> Packet for Udp<E> {
    type Envelope = E;
    type Header = UdpHeader;

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

        Ok(Udp {
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

        Ok(Udp {
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
        let len = self.len() as u16;
        self.set_length(len);
        self.compute_checksum();
        self.envelope_mut().cascade();
    }

    #[inline]
    fn deparse(self) -> Self::Envelope {
        self.envelope
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use dpdk_test;
    use packets::ip::v4::Ipv4;
    use packets::{Ethernet, RawPacket};
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[rustfmt::skip]
    pub const UDP_PACKET: [u8; 52] = [
        // ** ethernet header
        0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
        0x08, 0x00,
        // ** IPv4 header
        0x45, 0x00,
        // IPv4 payload length
        0x00, 0x26,
        // ident = 43849, flags = 4, frag_offset = 0
        0xab, 0x49, 0x40, 0x00,
        // ttl = 255, protocol = UDP, checksum = 0xf700
        0xff, 0x11, 0xf7, 0x00,
        // src = 139.133.217.110
        0x8b, 0x85, 0xd9, 0x6e,
        // dst = 139.133.233.2
        0x8b, 0x85, 0xe9, 0x02,
        // ** UDP header
        // src_port = 39376, dst_port = 1087
        0x99, 0xd0, 0x04, 0x3f,
        // UDP length = 18, checksum = 0x7228
        0x00, 0x12, 0x72, 0x28,
        // ** UDP payload
        0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x68, 0x65, 0x6c, 0x6c, 0x6f
    ];

    #[test]
    fn size_of_udp_header() {
        assert_eq!(8, UdpHeader::size());
    }

    #[test]
    fn parse_udp_packet() {
        dpdk_test! {
            let packet = RawPacket::from_bytes(&UDP_PACKET).unwrap();
            let ethernet = packet.parse::<Ethernet>().unwrap();
            let ipv4 = ethernet.parse::<Ipv4>().unwrap();
            let udp = ipv4.parse::<Udp<Ipv4>>().unwrap();

            assert_eq!(39376, udp.src_port());
            assert_eq!(1087, udp.dst_port());
            assert_eq!(18, udp.length());
            assert_eq!(0x7228, udp.checksum());
        }
    }

    #[test]
    fn udp_flow_v4() {
        dpdk_test! {
            let packet = RawPacket::from_bytes(&UDP_PACKET).unwrap();
            let ethernet = packet.parse::<Ethernet>().unwrap();
            let ipv4 = ethernet.parse::<Ipv4>().unwrap();
            let udp = ipv4.parse::<Udp<Ipv4>>().unwrap();
            let flow = udp.flow();

            assert_eq!("139.133.217.110", flow.src_ip().to_string());
            assert_eq!("139.133.233.2", flow.dst_ip().to_string());
            assert_eq!(39376, flow.src_port());
            assert_eq!(1087, flow.dst_port());
            assert_eq!(ProtocolNumbers::Udp, flow.protocol());
        }
    }

    #[test]
    fn set_src_dst_ip() {
        dpdk_test! {
            let packet = RawPacket::from_bytes(&UDP_PACKET).unwrap();
            let ethernet = packet.parse::<Ethernet>().unwrap();
            let ipv4 = ethernet.parse::<Ipv4>().unwrap();
            let mut udp = ipv4.parse::<Udp<Ipv4>>().unwrap();

            let old_checksum = udp.checksum();
            let new_ip = Ipv4Addr::new(10, 0, 0, 0);
            assert!(udp.set_src_ip(new_ip.into()).is_ok());
            assert!(udp.checksum() != old_checksum);
            assert_eq!(new_ip.to_string(), udp.envelope().src().to_string());

            let old_checksum = udp.checksum();
            let new_ip = Ipv4Addr::new(20, 0, 0, 0);
            assert!(udp.set_dst_ip(new_ip.into()).is_ok());
            assert!(udp.checksum() != old_checksum);
            assert_eq!(new_ip.to_string(), udp.envelope().dst().to_string());

            // can't set v6 addr on a v4 packet
            assert!(udp.set_src_ip(Ipv6Addr::UNSPECIFIED.into()).is_err());
        }
    }

    #[test]
    fn compute_checksum() {
        dpdk_test! {
            let packet = RawPacket::from_bytes(&UDP_PACKET).unwrap();
            let ethernet = packet.parse::<Ethernet>().unwrap();
            let ipv4 = ethernet.parse::<Ipv4>().unwrap();
            let mut udp = ipv4.parse::<Udp<Ipv4>>().unwrap();

            let expected = udp.checksum();
            // no payload change but force a checksum recompute anyway
            udp.cascade();
            assert_eq!(expected, udp.checksum());
        }
    }

    #[test]
    fn push_udp_packet() {
        dpdk_test! {
            let packet = RawPacket::new().unwrap();
            let ethernet = packet.push::<Ethernet>().unwrap();
            let ipv4 = ethernet.push::<Ipv4>().unwrap();
            let udp = ipv4.push::<Udp<Ipv4>>().unwrap();

            assert_eq!(UdpHeader::size(), udp.len());
        }
    }
}
