use super::super::io;
use std::fmt;

/// IP header.
#[derive(Debug)]
#[repr(C)]
pub struct IpHeader {
    version_ihl: u8,
    dscp_ecn: u8,
    pub len: u16,
    pub id: u16,
    pub flags_fragment: u16,
    pub ttl: u8,
    pub protocol: u8,
    pub csum: u16,
    pub src: [u8; 4],
    pub dst: [u8; 4]
}

impl fmt::Display for IpHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}.{} > {}.{}.{}.{} len: {} ttl: {} proto: {} csum: {}",
               self.src[0], self.src[1], self.src[2], self.src[3],
               self.dst[0], self.dst[1], self.dst[2], self.dst[3],
               u16::from_be(self.len), self.ttl, self.protocol, self.csum)
    }
}

impl IpHeader {
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

impl io::ConstFromU8 for IpHeader {
    #[inline]
    fn from_u8<'a>(data: *const u8) -> &'a Self {
        let typecast = data as *const IpHeader;
        unsafe {&*typecast}
    }
}

impl io::MutFromU8 for IpHeader {
    #[inline]
    fn from_u8<'a>(data: *mut u8) -> &'a mut Self {
        let typecast = data as *mut IpHeader;
        unsafe {&mut *typecast}
    }
}

impl io::EndOffset for IpHeader {
    #[inline]
    fn offset(&self) -> usize {
        self.header_len() as usize
    }
}
