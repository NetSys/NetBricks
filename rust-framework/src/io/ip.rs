use super::interface;
//use std::fmt;

/// IP header.
#[derive(Debug)]
#[repr(C)]
pub struct IpHeader {
    version_ihl: u8,
    dscp_ecn: u8,
    pub len: u16,
    pub ttl: u16,
    pub protocol: u16,
    pub csum: u16,
    pub source: u32,
    pub dest: u32
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

impl interface::ConstFromU8 for IpHeader {
    #[inline]
    fn from_u8<'a>(data: *const u8) -> &'a Self {
        let typecast = data as *const IpHeader;
        unsafe {&*typecast}
    }
}

impl interface::MutFromU8 for IpHeader {
    #[inline]
    fn from_u8<'a>(data: *mut u8) -> &'a mut Self {
        let typecast = data as *mut IpHeader;
        unsafe {&mut *typecast}
    }
}

impl interface::EndOffset for IpHeader {
    #[inline]
    fn offset(&self) -> usize {
        self.header_len() as usize
    }
}
