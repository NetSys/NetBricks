use common::Result;
use native::mbuf::MBuf;
use packets::ip::{Flow, IpPacket, ProtocolNumbers};
use packets::{buffer, checksum, Fixed, Header, Packet};
use std::fmt;
use std::net::IpAddr;

/*  From https://tools.ietf.org/html/rfc793#section-3.1
    TCP Header Format

     0                   1                   2                   3
     0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |          Source Port          |       Destination Port        |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |                        Sequence Number                        |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |                    Acknowledgment Number                      |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |  Data |     |N|C|E|U|A|P|R|S|F|                               |
    | Offset| Res |S|W|C|R|C|S|S|Y|I|            Window             |
    |       |     | |R|E|G|K|H|T|N|N|                               |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |           Checksum            |         Urgent Pointer        |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |                    Options                    |    Padding    |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |                             data                              |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

    Source Port:  16 bits
        The source port number.

    Destination Port:  16 bits
        The destination port number.

    Sequence Number:  32 bits
        The sequence number of the first data octet in this segment (except
        when SYN is present). If SYN is present the sequence number is the
        initial sequence number (ISN) and the first data octet is ISN+1.

    Acknowledgment Number:  32 bits
        If the ACK control bit is set this field contains the value of the
        next sequence number the sender of the segment is expecting to
        receive.  Once a connection is established this is always sent.

    Data Offset:  4 bits
        The number of 32 bit words in the TCP Header.  This indicates where
        the data begins.  The TCP header (even one including options) is an
        integral number of 32 bits long.

    Control Bits:  9 bits (from left to right):
        NS:   ECN-nonce nonce sum (see https://tools.ietf.org/html/rfc3540)
        CWR:  Congestion Window Reduced flag (see https://tools.ietf.org/html/rfc3168)
        ECE:  ECN-Echo flag (see https://tools.ietf.org/html/rfc3168)
        URG:  Urgent Pointer field significant
        ACK:  Acknowledgment field significant
        PSH:  Push Function
        RST:  Reset the connection
        SYN:  Synchronize sequence numbers
        FIN:  No more data from sender

    Window:  16 bits
        The number of data octets beginning with the one indicated in the
        acknowledgment field which the sender of this segment is willing to
        accept.

    Checksum:  16 bits
        The checksum field is the 16 bit one's complement of the one's
        complement sum of all 16 bit words in the header and text.  If a
        segment contains an odd number of header and text octets to be
        checksummed, the last octet is padded on the right with zeros to
        form a 16 bit word for checksum purposes.  The pad is not
        transmitted as part of the segment.  While computing the checksum,
        the checksum field itself is replaced with zeros.

        The checksum also covers a 96 bit pseudo header conceptually
        prefixed to the TCP header.  This pseudo header contains the Source
        Address, the Destination Address, the Protocol, and TCP length.
        This gives the TCP protection against misrouted segments.  This
        information is carried in the Internet Protocol and is transferred
        across the TCP/Network interface in the arguments or results of
        calls by the TCP on the IP.

                    +--------+--------+--------+--------+
                    |           Source Address          |
                    +--------+--------+--------+--------+
                    |         Destination Address       |
                    +--------+--------+--------+--------+
                    |  zero  |  PTCL  |    TCP Length   |
                    +--------+--------+--------+--------+

        The TCP Length is the TCP header length plus the data length in
        octets (this is not an explicitly transmitted quantity, but is
        computed), and it does not count the 12 octets of the pseudo
        header.

    Urgent Pointer:  16 bits
        This field communicates the current value of the urgent pointer as a
        positive offset from the sequence number in this segment.  The
        urgent pointer points to the sequence number of the octet following
        the urgent data.  This field is only be interpreted in segments with
        the URG control bit set.

    Options:  variable
        Options may occupy space at the end of the TCP header and are a
        multiple of 8 bits in length.  All options are included in the
        checksum.
*/

/// TCP header
///
/// The header only include the fixed portion of the TCP header.
/// Options are parsed separately.
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct TcpHeader {
    src_port: u16,
    dst_port: u16,
    seq_no: u32,
    ack_no: u32,
    offset_to_ns: u8,
    flags: u8,
    window: u16,
    checksum: u16,
    urgent_pointer: u16,
}

impl Default for TcpHeader {
    fn default() -> TcpHeader {
        TcpHeader {
            src_port: 0,
            dst_port: 0,
            seq_no: 0,
            ack_no: 0,
            offset_to_ns: 5 << 4,
            flags: 0,
            window: 0,
            checksum: 0,
            urgent_pointer: 0,
        }
    }
}

impl Header for TcpHeader {}

const CWR: u8 = 0b1000_0000;
const ECE: u8 = 0b0100_0000;
const URG: u8 = 0b0010_0000;
const ACK: u8 = 0b0001_0000;
const PSH: u8 = 0b0000_1000;
const RST: u8 = 0b0000_0100;
const SYN: u8 = 0b0000_0010;
const FIN: u8 = 0b0000_0001;

/// TCP packet
#[derive(Debug)]
pub struct Tcp<E: IpPacket> {
    envelope: E,
    mbuf: *mut MBuf,
    offset: usize,
    header: *mut TcpHeader,
}

impl<E: IpPacket> Tcp<E> {
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
    pub fn seq_no(&self) -> u32 {
        u32::from_be(self.header().seq_no)
    }

    #[inline]
    pub fn set_seq_no(&mut self, seq_no: u32) {
        self.header_mut().seq_no = u32::to_be(seq_no);
    }

    #[inline]
    pub fn ack_no(&self) -> u32 {
        u32::from_be(self.header().ack_no)
    }

    #[inline]
    pub fn set_ack_no(&mut self, ack_no: u32) {
        self.header_mut().ack_no = u32::to_be(ack_no);
    }

    #[inline]
    pub fn data_offset(&self) -> u8 {
        (self.header().offset_to_ns & 0xf0) >> 4
    }

    // TODO: support tcp header options
    #[allow(dead_code)]
    #[inline]
    fn set_data_offset(&mut self, data_offset: u8) {
        self.header_mut().offset_to_ns = (self.header().offset_to_ns & 0x0f) | (data_offset << 4);
    }

    #[inline]
    pub fn ns(&self) -> bool {
        (self.header().offset_to_ns & 0x01) != 0
    }

    #[inline]
    pub fn set_ns(&mut self) {
        self.header_mut().offset_to_ns |= 0x01;
    }

    #[inline]
    pub fn unset_ns(&mut self) {
        self.header_mut().offset_to_ns &= !0x1;
    }

    #[inline]
    pub fn cwr(&self) -> bool {
        (self.header().flags & CWR) != 0
    }

    #[inline]
    pub fn set_cwr(&mut self) {
        self.header_mut().flags |= CWR;
    }

    #[inline]
    pub fn unset_cwr(&mut self) {
        self.header_mut().flags &= !CWR;
    }

    #[inline]
    pub fn ece(&self) -> bool {
        (self.header().flags & ECE) != 0
    }

    #[inline]
    pub fn set_ece(&mut self) {
        self.header_mut().flags |= ECE;
    }

    #[inline]
    pub fn unset_ece(&mut self) {
        self.header_mut().flags &= !ECE;
    }

    #[inline]
    pub fn urg(&self) -> bool {
        (self.header().flags & URG) != 0
    }

    #[inline]
    pub fn set_urg(&mut self) {
        self.header_mut().flags |= URG;
    }

    #[inline]
    pub fn unset_urg(&mut self) {
        self.header_mut().flags &= !URG;
    }

    #[inline]
    pub fn ack(&self) -> bool {
        (self.header().flags & ACK) != 0
    }

    #[inline]
    pub fn set_ack(&mut self) {
        self.header_mut().flags |= ACK;
    }

    #[inline]
    pub fn unset_ack(&mut self) {
        self.header_mut().flags &= !ACK;
    }

    #[inline]
    pub fn psh(&self) -> bool {
        (self.header().flags & PSH) != 0
    }

    #[inline]
    pub fn set_psh(&mut self) {
        self.header_mut().flags |= PSH;
    }

    #[inline]
    pub fn unset_psh(&mut self) {
        self.header_mut().flags &= !PSH;
    }

    #[inline]
    pub fn rst(&self) -> bool {
        (self.header().flags & RST) != 0
    }

    #[inline]
    pub fn set_rst(&mut self) {
        self.header_mut().flags |= RST;
    }

    #[inline]
    pub fn unset_rst(&mut self) {
        self.header_mut().flags &= !RST;
    }

    #[inline]
    pub fn syn(&self) -> bool {
        (self.header().flags & SYN) != 0
    }

    #[inline]
    pub fn set_syn(&mut self) {
        self.header_mut().flags |= SYN;
    }

    #[inline]
    pub fn unset_syn(&mut self) {
        self.header_mut().flags &= !SYN;
    }

    #[inline]
    pub fn fin(&self) -> bool {
        (self.header().flags & FIN) != 0
    }

    #[inline]
    pub fn set_fin(&mut self) {
        self.header_mut().flags |= FIN;
    }

    #[inline]
    pub fn unset_fin(&mut self) {
        self.header_mut().flags &= !FIN;
    }

    #[inline]
    pub fn window(&self) -> u16 {
        u16::from_be(self.header().window)
    }

    #[inline]
    pub fn set_window(&mut self, window: u16) {
        self.header_mut().window = u16::to_be(window);
    }

    #[inline]
    pub fn checksum(&self) -> u16 {
        u16::from_be(self.header().checksum)
    }

    #[inline]
    fn set_checksum(&mut self, checksum: u16) {
        self.header_mut().checksum = u16::to_be(checksum);
    }

    #[inline]
    pub fn urgent_pointer(&self) -> u16 {
        u16::from_be(self.header().urgent_pointer)
    }

    #[inline]
    pub fn set_urgent_pointer(&mut self, urgent_pointer: u16) {
        self.header_mut().urgent_pointer = u16::to_be(urgent_pointer);
    }

    #[inline]
    pub fn flow(&self) -> Flow {
        Flow::new(
            self.envelope().src(),
            self.envelope().dst(),
            self.src_port(),
            self.dst_port(),
            ProtocolNumbers::Tcp,
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
        self.set_checksum(0);

        if let Ok(data) = buffer::read_slice(self.mbuf, self.offset, self.len()) {
            let data = unsafe { &(*data) };
            let pseudo_header_sum = self
                .envelope()
                .pseudo_header(data.len() as u16, ProtocolNumbers::Tcp)
                .sum();
            let checksum = checksum::compute(pseudo_header_sum, data);
            self.set_checksum(checksum);
        } else {
            // we are reading till the end of buffer, should never run out
            unreachable!()
        }
    }
}

impl<E: IpPacket> fmt::Display for Tcp<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "src_port: {}, dst_port: {}, seq_no: {}, ack_no: {}, data_offset: {}, window: {}, checksum {}, urgent: {}, \
            NS: {}, CWR: {}, ECE: {}, URG: {}, ACK: {}, PSH: {}, RST: {}, SYN: {}, FIN: {}",
            self.src_port(),
            self.dst_port(),
            self.seq_no(),
            self.ack_no(),
            self.data_offset(),
            self.window(),
            self.checksum(),
            self.urgent_pointer(),
            self.ns(),
            self.cwr(),
            self.ece(),
            self.urg(),
            self.ack(),
            self.psh(),
            self.rst(),
            self.syn(),
            self.fin()
        )
    }
}

impl<E: IpPacket> Packet for Tcp<E> {
    type Envelope = E;
    type Header = TcpHeader;

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

        Ok(Tcp {
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

        Ok(Tcp {
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
    use packets::ip::v6::{Ipv6, SegmentRouting};
    use packets::{Ethernet, RawPacket};
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[rustfmt::skip]
    pub const TCP_PACKET: [u8; 58] = [
        // ** ethernet header
        0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
        0x08, 0x00,
        // ** IPv4 header
        0x45, 0x00,
        // IPv4 payload length
        0x00, 0x2c,
        // ident = 2232, flags = 4, frag_offset = 0
        0x08, 0xb8, 0x40, 0x00,
        // ttl = 255, protocol = TCP, checksum = 0x9997
        0xff, 0x06, 0x99, 0x97,
        // src = 139.133.217.110
        0x8b, 0x85, 0xd9, 0x6e,
        // dst = 139.133.233.2
        0x8b, 0x85, 0xe9, 0x02,
        // ** TCP header
        // src_port = 36869, dst_port = 23
        0x90, 0x05, 0x00, 0x17,
        // seq_no = 1913975060
        0x72, 0x14, 0xf1, 0x14,
        // ack_no = 0
        0x00, 0x00, 0x00, 0x00,
        // data_offset = 6, flags = 0x02
        0x60, 0x02,
        // window = 8760, checksum = 0xa92c, urgent = 0
        0x22, 0x38, 0xa9, 0x2c, 0x00, 0x00,
        // options
        0x02, 0x04, 0x05, 0xb4
    ];

    #[test]
    fn size_of_tcp_header() {
        assert_eq!(20, TcpHeader::size());
    }

    #[test]
    fn parse_tcp_packet() {
        dpdk_test! {
            let packet = RawPacket::from_bytes(&TCP_PACKET).unwrap();
            let ethernet = packet.parse::<Ethernet>().unwrap();
            let ipv4 = ethernet.parse::<Ipv4>().unwrap();
            let tcp = ipv4.parse::<Tcp<Ipv4>>().unwrap();

            assert_eq!(36869, tcp.src_port());
            assert_eq!(23, tcp.dst_port());
            assert_eq!(1_913_975_060, tcp.seq_no());
            assert_eq!(0, tcp.ack_no());
            assert_eq!(6, tcp.data_offset());
            assert_eq!(8760, tcp.window());
            assert_eq!(0xa92c, tcp.checksum());
            assert_eq!(0, tcp.urgent_pointer());
            assert!(!tcp.ns());
            assert!(!tcp.cwr());
            assert!(!tcp.ece());
            assert!(!tcp.urg());
            assert!(!tcp.ack());
            assert!(!tcp.psh());
            assert!(!tcp.rst());
            assert!(tcp.syn());
            assert!(!tcp.fin());
        }
    }

    #[test]
    fn tcp_flow_v4() {
        dpdk_test! {
            let packet = RawPacket::from_bytes(&TCP_PACKET).unwrap();
            let ethernet = packet.parse::<Ethernet>().unwrap();
            let ipv4 = ethernet.parse::<Ipv4>().unwrap();
            let tcp = ipv4.parse::<Tcp<Ipv4>>().unwrap();
            let flow = tcp.flow();

            assert_eq!("139.133.217.110", flow.src_ip().to_string());
            assert_eq!("139.133.233.2", flow.dst_ip().to_string());
            assert_eq!(36869, flow.src_port());
            assert_eq!(23, flow.dst_port());
            assert_eq!(ProtocolNumbers::Tcp, flow.protocol());
        }
    }

    #[test]
    fn tcp_flow_v6() {
        use packets::ip::v6::srh::tests::SRH_PACKET;

        dpdk_test! {
            let packet = RawPacket::from_bytes(&SRH_PACKET).unwrap();
            let ethernet = packet.parse::<Ethernet>().unwrap();
            let ipv6 = ethernet.parse::<Ipv6>().unwrap();
            let srh = ipv6.parse::<SegmentRouting<Ipv6>>().unwrap();
            let tcp = srh.parse::<Tcp<SegmentRouting<Ipv6>>>().unwrap();
            let flow = tcp.flow();

            assert_eq!("2001:db8:85a3::1", flow.src_ip().to_string());
            assert_eq!("2001:db8:85a3::8a2e:370:7334", flow.dst_ip().to_string());
            assert_eq!(3464, flow.src_port());
            assert_eq!(1024, flow.dst_port());
            assert_eq!(ProtocolNumbers::Tcp, flow.protocol());
        }
    }

    #[test]
    fn set_src_dst_ip() {
        dpdk_test! {
            let packet = RawPacket::from_bytes(&TCP_PACKET).unwrap();
            let ethernet = packet.parse::<Ethernet>().unwrap();
            let ipv4 = ethernet.parse::<Ipv4>().unwrap();
            let mut tcp = ipv4.parse::<Tcp<Ipv4>>().unwrap();

            let old_checksum = tcp.checksum();
            let new_ip = Ipv4Addr::new(10, 0, 0, 0);
            assert!(tcp.set_src_ip(new_ip.into()).is_ok());
            assert!(tcp.checksum() != old_checksum);
            assert_eq!(new_ip.to_string(), tcp.envelope().src().to_string());

            let old_checksum = tcp.checksum();
            let new_ip = Ipv4Addr::new(20, 0, 0, 0);
            assert!(tcp.set_dst_ip(new_ip.into()).is_ok());
            assert!(tcp.checksum() != old_checksum);
            assert_eq!(new_ip.to_string(), tcp.envelope().dst().to_string());

            // can't set v6 addr on a v4 packet
            assert!(tcp.set_src_ip(Ipv6Addr::UNSPECIFIED.into()).is_err());
        }
    }

    #[test]
    fn compute_checksum() {
        dpdk_test! {
            let packet = RawPacket::from_bytes(&TCP_PACKET).unwrap();
            let ethernet = packet.parse::<Ethernet>().unwrap();
            let ipv4 = ethernet.parse::<Ipv4>().unwrap();
            let mut tcp = ipv4.parse::<Tcp<Ipv4>>().unwrap();

            let expected = tcp.checksum();
            // no payload change but force a checksum recompute anyway
            tcp.cascade();
            assert_eq!(expected, tcp.checksum());
        }
    }

    #[test]
    fn push_tcp_packet() {
        dpdk_test! {
            let packet = RawPacket::new().unwrap();
            let ethernet = packet.push::<Ethernet>().unwrap();
            let ipv4 = ethernet.push::<Ipv4>().unwrap();
            let tcp = ipv4.push::<Tcp<Ipv4>>().unwrap();

            assert_eq!(TcpHeader::size(), tcp.len());
            assert_eq!(5, tcp.data_offset());
        }
    }
}
