use super::super::io;
use std::fmt;
use std::net::Ipv4Addr;
use std::convert::From;

/// IP header.
#[derive(Debug)]
#[repr(C, packed)]
pub struct IpHeader {
    version_ihl: u8, // 1
    dscp_ecn: u8,    // 1
    pub len: u16,    // 2

    // 128 bits below
    pub id: u16,     // 2
    pub flags_fragment: u16, // 2

    pub ttl: u8,  // 1
    pub protocol: u8, // 1
    pub csum: u16, // 2

    pub src: u32, // 4
    pub dst: u32, // 4
}

impl fmt::Display for IpHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let src = Ipv4Addr::from(self.src);
        let dst = Ipv4Addr::from(self.dst);
        write!(f, "{} > {} len: {} ttl: {} proto: {} csum: {}",
               src, dst,
               u16::from_be(self.len), self.ttl, self.protocol, self.csum)
    }
}


impl io::EndOffset for IpHeader {
    #[inline]
    fn offset(&self) -> usize {
        self.header_len() as usize * 4
    }
}
impl IpHeader {
    pub fn new() -> IpHeader {
        IpHeader{version_ihl: 0, dscp_ecn: 0, len: 0, id: 0, flags_fragment: 0, 
            ttl: 0, protocol: 0, csum: 0, src: 0, dst: 0}
    }

    #[inline]
    pub fn version(&self) -> u8 {
        (self.version_ihl & 0xf0) >> 4
    }

    #[inline]
    pub fn header_len(&self) -> u8 {
        (self.version_ihl & 0x0f)
    }

    #[inline]
    pub fn dscp(&self) -> u8 {
        self.dscp_ecn >> 2
    }

    #[inline]
    pub fn ecn(&self) -> u8 {
        self.dscp_ecn & 0x3
    }

    #[inline]
    pub fn set_version(&mut self, version: u8) {
        self.version_ihl = (self.version_ihl & 0x0f) | ((version & 0x0f) << 4);
    }

    #[inline]
    pub fn set_header_len(&mut self, len: u8) {
        self.version_ihl = (self.version_ihl & 0xf0) | (len & 0x0f);
    }

    #[inline]
    pub fn set_dscp(&mut self, dscp: u8) {
        self.dscp_ecn = (self.dscp_ecn & 0x3) | (dscp << 2);
    }

    #[inline]
    pub fn set_ecn(&mut self, ecn: u8) {
        self.dscp_ecn = (self.dscp_ecn & !0x3) | (ecn & 0x3);
    }
}
