use common::Result;
use native::zcsi::MBuf;
use packets::{buffer, Fixed, Header, Packet, RawPacket};
use std::fmt;

/* Ethernet Type II Frame

   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   |  Dst MAC  |  Src MAC  |Typ|             Payload               |
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+                                   +
   |                                                               |
   |                                                               |
   |                                                               |
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

   Destination MAC      48-bit MAC address of the originator of the
                        packet.

   Source MAC           48-bit MAC address of the intended recipient of
                        the packet.

   Ether Type           16-bit indicator. Identifies which protocol is
                        encapsulated in the payload of the frame.
*/

/// MAC address
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
#[repr(C, packed)]
pub struct MacAddr(pub [u8; 6]);

impl MacAddr {
    pub const UNSPECIFIED: Self = MacAddr([0, 0, 0, 0, 0, 0]);

    pub fn new(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8) -> Self {
        MacAddr([a, b, c, d, e, f])
    }

    pub fn new_from_slice(slice: &[u8]) -> Self {
        MacAddr([slice[0], slice[1], slice[2], slice[3], slice[4], slice[5]])
    }
}

impl fmt::Display for MacAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

/// The protocol type in the ethernet packet payload
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
#[repr(C, packed)]
pub struct EtherType(pub u16);

impl EtherType {
    pub fn new(value: u16) -> Self {
        EtherType(value)
    }
}

/// Supported ethernet payload protocol types
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub mod EtherTypes {
    use super::EtherType;

    // Internet Protocol version 4
    pub const Ipv4: EtherType = EtherType(0x0800);
    // Internet Protocol version 6
    pub const Ipv6: EtherType = EtherType(0x86DD);
}

impl fmt::Display for EtherType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                &EtherTypes::Ipv4 => "IPv4".to_string(),
                &EtherTypes::Ipv6 => "IPv6".to_string(),
                _ => format!("0x{:04x}", self.0),
            }
        )
    }
}

/// Ethernet header
#[derive(Default, Debug)]
#[repr(C, packed)]
pub struct EthernetHeader {
    dst: MacAddr,
    src: MacAddr,
    ether_type: u16,
}

impl Header for EthernetHeader {}

/// Ethernet packet
pub struct Ethernet {
    envelope: RawPacket,
    mbuf: *mut MBuf,
    offset: usize,
    header: *mut EthernetHeader,
}

impl Ethernet {
    #[inline]
    pub fn src(&self) -> MacAddr {
        self.header().src
    }

    #[inline]
    pub fn set_src(&mut self, src: MacAddr) {
        self.header().src = src
    }

    #[inline]
    pub fn dst(&self) -> MacAddr {
        self.header().dst
    }

    #[inline]
    pub fn set_dst(&mut self, dst: MacAddr) {
        self.header().dst = dst
    }

    #[inline]
    pub fn ether_type(&self) -> EtherType {
        EtherType::new(u16::from_be(self.header().ether_type))
    }

    #[inline]
    pub fn set_ether_type(&mut self, ether_type: EtherType) {
        self.header().ether_type = u16::to_be(ether_type.0)
    }

    #[inline]
    pub fn swap_addresses(&mut self) {
        let src = self.src();
        let dst = self.dst();
        self.set_src(dst);
        self.set_dst(src);
    }
}

impl fmt::Display for Ethernet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} > {}, ether_type: {}",
            self.src(),
            self.dst(),
            self.ether_type()
        )
    }
}

impl Packet for Ethernet {
    type Header = EthernetHeader;
    type Envelope = RawPacket;

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

        Ok(Ethernet {
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

        Ok(Ethernet {
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
    fn cascade(&self) {
        self.envelope().cascade();
    }

    #[inline]
    fn deparse(self) -> Self::Envelope {
        self.envelope
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dpdk_test;

    #[test]
    fn size_of_ethernet_header() {
        assert_eq!(14, EthernetHeader::size());
    }

    #[test]
    fn mac_addr_to_string() {
        assert_eq!(
            "00:00:00:00:00:00",
            MacAddr::new(0, 0, 0, 0, 0, 0).to_string()
        );
        assert_eq!(
            "ff:ff:ff:ff:ff:ff",
            MacAddr::new(255, 255, 255, 255, 255, 255).to_string()
        );
        assert_eq!(
            "12:34:56:ab:cd:ef",
            MacAddr::new(0x12, 0x34, 0x56, 0xAB, 0xCD, 0xEF).to_string()
        );
    }

    #[test]
    fn ether_type_to_string() {
        assert_eq!("IPv4", EtherTypes::Ipv4.to_string());
        assert_eq!("IPv6", EtherTypes::Ipv6.to_string());
        assert_eq!("0x0000", EtherType::new(0).to_string());
    }

    #[test]
    fn parse_ethernet_packet() {
        use packets::udp::tests::UDP_PACKET;

        dpdk_test! {
            let packet = RawPacket::from_bytes(&UDP_PACKET).unwrap();
            let ethernet = packet.parse::<Ethernet>().unwrap();

            assert_eq!("00:00:00:00:00:01", ethernet.dst().to_string());
            assert_eq!("00:00:00:00:00:02", ethernet.src().to_string());
            assert_eq!(EtherTypes::Ipv4, ethernet.ether_type());
        }
    }

    #[test]
    fn swap_addresses() {
        use packets::udp::tests::UDP_PACKET;

        dpdk_test! {
            let packet = RawPacket::from_bytes(&UDP_PACKET).unwrap();
            let mut ethernet = packet.parse::<Ethernet>().unwrap();
            ethernet.swap_addresses();

            assert_eq!("00:00:00:00:00:02", ethernet.dst().to_string());
            assert_eq!("00:00:00:00:00:01", ethernet.src().to_string());
        }
    }

    #[test]
    fn push_ethernet_packet() {
        dpdk_test! {
            let packet = RawPacket::new().unwrap();
            let ethernet = packet.push::<Ethernet>().unwrap();

            assert_eq!(EthernetHeader::size(), ethernet.len());
        }
    }
}
