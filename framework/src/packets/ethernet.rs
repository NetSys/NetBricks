use native::zcsi::MBuf;
use std::fmt;
use packets::{Packet, Header, RawPacket};

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
                _ => format!("0x{:04x}", self.0)
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
    ether_type: u16
}

impl Header for EthernetHeader {
    fn size() -> usize {
        14
    }
}

/// Ethernet packet
pub struct Ethernet {
    envelope: RawPacket,
    mbuf: *mut MBuf,
    offset: usize,
    header: *mut EthernetHeader
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
}

impl fmt::Display for Ethernet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} > {}, ether_type: {}", self.src(), self.dst(), self.ether_type())
    }
}

impl Packet for Ethernet {
    type Header = EthernetHeader;
    type Envelope = RawPacket;

    #[inline]
    fn from_packet(envelope: Self::Envelope,
                   mbuf: *mut MBuf,
                   offset: usize,
                   header: *mut Self::Header) -> Self {
        Ethernet {
            envelope,
            mbuf,
            offset,
            header
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
        Self::Header::size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dpdk_test;

    #[rustfmt::skip]
    const UDP_PACKET: [u8; 52] = [
        // ** ethernet header
        0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
        0x08, 0x00,
        // ** IPv4 header
        0x45, 0x00,
        // payload length
        0x00, 0x26,
        0xab, 0x49, 0x40, 0x00,
        0xff, 0x11, 0xf7, 0x00,
        0x8b, 0x85, 0xd9, 0x6e,
        0x8b, 0x85, 0xe9, 0x02,
        // ** UDP header
        0x99, 0xd0, 0x04, 0x3f,
        0x00, 0x12, 0x72, 0x28,
        // ** UDP payload
        0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x68, 0x65, 0x6c, 0x6c, 0x6f
    ];

    #[test]
    fn mac_addr_to_string() {
        assert_eq!("00:00:00:00:00:00", MacAddr::new(0, 0, 0, 0, 0, 0).to_string());
        assert_eq!("ff:ff:ff:ff:ff:ff", MacAddr::new(255, 255, 255, 255, 255, 255).to_string());
        assert_eq!("12:34:56:ab:cd:ef", MacAddr::new(0x12, 0x34, 0x56, 0xAB, 0xCD, 0xEF).to_string());
    }

    #[test]
    fn ether_type_to_string() {
        assert_eq!("IPv4", EtherTypes::Ipv4.to_string());
        assert_eq!("IPv6", EtherTypes::Ipv6.to_string());
        assert_eq!("0x0000", EtherType::new(0).to_string());
    }

    #[test]
    fn parse_ethernet_packet() {
        dpdk_test! {
            let packet = RawPacket::from_bytes(&UDP_PACKET).unwrap();
            let ethernet = packet.parse::<Ethernet>().unwrap();

            assert_eq!("00:00:00:00:00:01", ethernet.dst().to_string());
            assert_eq!("00:00:00:00:00:02", ethernet.src().to_string());
            assert_eq!(EtherTypes::Ipv4, ethernet.ether_type());
        }
    }
}
