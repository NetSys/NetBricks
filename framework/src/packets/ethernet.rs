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

/// mac address
#[derive(Default, Debug, Copy, Clone)]
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

/// ethernet type
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
#[repr(C, packed)]
pub struct EtherType(pub u16);

impl EtherType {
    pub fn new(value: u16) -> Self {
        EtherType(value)
    }
}

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

/// ethernet header
#[derive(Default, Debug)]
#[repr(C, packed)]
pub struct MacHeader {
    dst: MacAddr,
    src: MacAddr,
    ether_type: u16
}

impl Header for MacHeader {
    fn size() -> usize {
        14
    }
}

/// ethernet packet
pub struct Ethernet {
    mbuf: *mut MBuf,
    offset: usize,
    header: *mut MacHeader,
    previous: RawPacket
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
        write!(f, "{} > {} [{}]", self.src(), self.dst(), self.ether_type())
    }
}

impl Packet for Ethernet {
    type Header = MacHeader;
    type PreviousPacket = RawPacket;

    #[inline]
    fn from_packet(previous: Self::PreviousPacket,
                   mbuf: *mut MBuf,
                   offset: usize,
                   header: *mut Self::Header) -> Self {
        Ethernet {
            previous,
            mbuf,
            offset,
            header
        }
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
    use tests::V6_BYTES;

    #[test]
    fn str_from_mac_addr() {
        assert_eq!(format!("{}", MacAddr::new(0, 0, 0, 0, 0, 0)), "00:00:00:00:00:00");
        assert_eq!(format!("{}", MacAddr::new(255, 255, 255, 255, 255, 255)), "ff:ff:ff:ff:ff:ff");
        assert_eq!(format!("{}", MacAddr::new(0x12, 0x34, 0x56, 0xAB, 0xCD, 0xEF)), "12:34:56:ab:cd:ef");
    }

    #[test]
    fn str_from_ether_type() {
        assert_eq!(format!("{}", EtherTypes::Ipv4), "IPv4");
        assert_eq!(format!("{}", EtherTypes::Ipv6), "IPv6");
        assert_eq!(format!("{}", EtherType::new(0)), "0x0000");
    }

    #[test]
    fn str_from_ethernet_packet() {
        dpdk_test! {
            let packet = RawPacket::from_bytes(&V6_BYTES).unwrap();
            let ethernet = packet.parse::<Ethernet>().unwrap();
            assert_eq!(format!("{}", ethernet), "00:00:00:00:00:02 > 00:00:00:00:00:01 [IPv6]");
        }
    }
}
