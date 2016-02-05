use super::super::io;
use std::fmt;
use std::net::Ipv4Addr;
use std::convert::From;
//use self::simd::x86::*;
//use self::simd::x86::avx::*;

/// IP header using SSE
//#[repr(C, packed)]
#[derive(Debug)]
#[repr(simd)]
pub struct IpHeader {
    version_to_len: u32,
    rest0: u32,
    rest1: u32,
    rest2: u32,
    rest3: u32,
}

impl fmt::Display for IpHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let src = Ipv4Addr::from(self.src());
        let dst = Ipv4Addr::from(self.dst());
        write!(f, "{} > {} len: {} ttl: {} proto: {} csum: {}",
               src, dst,
               u16::from_be(self.length()), self.ttl(), self.protocol(), self.csum())
    }
}


impl io::EndOffset for IpHeader {
    #[inline]
    fn offset(&self) -> usize {
        self.ihl() as usize * 4
    }
}

impl IpHeader {
    #[inline]
    pub fn new() -> IpHeader {
        IpHeader{version_to_len: 0, rest0:0, rest1:0, rest2: 0, rest3:0}
    }

    #[inline]
    pub fn src(&self) -> u32 {
        let ip = u32::from_be(self.rest2);
        ip
    }

    #[inline]
    pub fn set_src(&mut self, src: u32) {
        self.rest2 = u32::to_be(src)
    }

    #[inline]
    pub fn dst(&self) -> u32 {
        let ip = u32::from_be(self.rest3);
        ip
    }

    #[inline]
    pub fn set_dst(&mut self, dst: u32) {
        self.rest3 = u32::to_be(dst);
    }

    #[inline]
    pub fn ttl(&self) -> u8 {
        let ttlpcsum = self.rest1;
        (ttlpcsum & 0x000000ff) as u8
    }

    #[inline]
    pub fn set_ttl(&mut self, ttl: u8) {
        let ttlpcsum = self.rest1;
        let blanked = ttlpcsum & !0x000000ff;
        self.rest1 = blanked | (ttl as u32);
    }

    #[inline]
    pub fn protocol(&self) -> u8 {
        let ttlpcsum = self.rest1;
        ((ttlpcsum & 0xff00) >> 8) as u8
    }

    #[inline]
    pub fn set_protocol(&mut self, protocol: u8) {
        let ttlpcsum = self.rest1;
        let blanked = ttlpcsum & !0xff00;
        self.rest1 = blanked | ((protocol as u32) << 8);
    }

    #[inline]
    pub fn csum(&self) -> u16 {
        let ttlpcsum = self.rest1;
        ((ttlpcsum & 0xffff0000) >> 16) as u16
    }

    #[inline]
    pub fn set_csum(&mut self, csum: u16) {
        let ttlpcsum = self.rest1;
        let blanked = ttlpcsum & !0xffff0000;
        self.rest1 = blanked | ((u16::to_be(csum) as u32) << 16);
    }

    #[inline]
    pub fn id(&self) -> u16 {
        let id_flag_fragment = self.rest0;
        u16::from_be((id_flag_fragment & 0xffff) as u16)
    }

    #[inline]
    pub fn set_id(&mut self, id: u16) {
        let id_flag_fragment = self.rest0;
        let blanked = id_flag_fragment & !0xffff;
        self.rest0 = blanked | (u16::to_be(id) as u32);
    }

    #[inline]
    pub fn flags(&self) -> u8 {
        let id_flag_fragment = self.rest0;
        let flag_fragment = (id_flag_fragment >> 16) as u16;
        (flag_fragment & 0x7) as u8
    }

    #[inline]
    pub fn set_flags(&mut self, flags: u8) {
        let flags_correct = (flags & 0x7) as u32; // Remove any extra bits because that would suck.
        let flags_mask = !0x70000;
        let id_flag_fragment = self.rest0;
        let blanked = id_flag_fragment & flags_mask;
        self.rest0 = blanked | (flags_correct << 16);
    }

    #[inline]
    pub fn fragment_offset(&self) -> u16 {
        let id_flag_fragment = self.rest0;
        let flag_fragment = (id_flag_fragment & 0xffff) as u16;
        u16::from_be(((flag_fragment & !0xe) >> 3))
    }

    #[inline]
    pub fn set_fragment_offset(&mut self, offset: u16) {
        let offset_correct = u16::to_be(offset & 0x1fff) as u32;
        let offset_shifted = offset_correct << 19;
        let blanked = self.rest0 & 0xfff80000;
        self.rest0 = blanked | offset_shifted;
    }

    #[inline]
    pub fn version(&self) -> u8 {
        let vihl = (self.version_to_len & 0xf) as u8;
        vihl
    }

    #[inline]
    pub fn set_version(&mut self, version: u8) {
        self.version_to_len = (self.version_to_len & !0xf) | ((version & 0xf) as u32);
    }

    #[inline]
    pub fn ihl(&self) -> u8 {
        let ihl = (self.version_to_len & 0xf0) as u8;
        ihl >> 4
    }

    #[inline]
    pub fn set_ihl(&mut self, ihl: u8) {
        self.version_to_len = (self.version_to_len & !0xf0) | (((ihl & 0xf) as u32) << 4);
    }

    #[inline]
    pub fn dscp(&self) -> u8 {
        let dscp_ecn = ((self.version_to_len & 0x3f00) >> 8) as u8;
        dscp_ecn
    }

    #[inline]
    pub fn set_dscp(&mut self, dscp: u8) {
        self.version_to_len = (self.version_to_len & !0x3f00) | (((dscp & 0x3f) as u32) << 8);
    }

    #[inline]
    pub fn ecn(&self) -> u8 {
        let dscp_ecn = ((self.version_to_len & 0xf0) >> 8) as u8;
        (dscp_ecn & !0x3f) >> 6
    }

    #[inline]
    pub fn set_ecn(&mut self, ecn: u8) {
        self.version_to_len = (self.version_to_len & !0xc000) | (((ecn & 0x03) as u32) << 14); 
    }

    #[inline]
    pub fn length(&self) -> u16 {
        u16::from_be(((self.version_to_len & 0xff00) >> 16) as u16)
    }

    #[inline]
    pub fn set_length(&mut self, len: u16) {
        self.version_to_len = (self.version_to_len & !0xffff0000) | ((u16::to_be(len) as u32) << 16);
    }

    // FIXME: Make sure this uses SIMD
    #[inline]
    pub fn apply(&mut self, from: &Self) {
        self.version_to_len = from.version_to_len;
        self.rest0 = from.rest0;
        self.rest1 = from.rest1;
        self.rest2 = from.rest2;
        self.rest3 = from.rest3;
    }
}
