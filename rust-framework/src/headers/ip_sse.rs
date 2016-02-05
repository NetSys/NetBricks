extern crate simd;
use super::super::io;
use std::fmt;
use self::simd::*;
use std::net::Ipv4Addr;
use std::convert::From;
//use self::simd::x86::*;
//use self::simd::x86::avx::*;

/// IP header using SSE
#[derive(Debug)]
#[repr(C, packed)]
pub struct IpHeaderSse {
    pub version_to_len: u32,
    pub rest: u32x4,
}

impl fmt::Display for IpHeaderSse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let src = Ipv4Addr::from(self.src());
        let dst = Ipv4Addr::from(self.dst());
        write!(f, "{} > {} len: {} ttl: {} proto: {} csum: {}",
               src, dst,
               u16::from_be(self.length()), self.ttl(), self.protocol(), self.csum())
    }
}


impl io::EndOffset for IpHeaderSse {
    #[inline]
    fn offset(&self) -> usize {
        //self.ihl() as usize * 4
        20
    }
}

impl IpHeaderSse {
    #[inline]
    pub fn new() -> IpHeaderSse {
        IpHeaderSse{version_to_len: 0, rest: u32x4::new(0, 0, 0, 0), align: []}
    }

    #[inline]
    pub fn src(&self) -> u32 {
        let ip = u32::from_be(self.rest.extract(2));
        ip
    }

    #[inline]
    pub fn set_src(&mut self, src: u32) {
        self.rest = self.rest.replace(2, u32::to_be(src))
    }

    #[inline]
    pub fn dst(&self) -> u32 {
        let ip = u32::from_be(self.rest.extract(3));
        ip
    }

    #[inline]
    pub fn set_dst(&mut self, dst: u32) {
        self.rest = self.rest.replace(3, u32::to_be(dst));
    }

    #[inline]
    pub fn ttl(&self) -> u8 {
        let ttlpcsum = self.rest.extract(1);
        (ttlpcsum & 0x000000ff) as u8
    }

    #[inline]
    pub fn set_ttl(&mut self, ttl: u8) {
        let ttlpcsum = self.rest.extract(1);
        let blanked = ttlpcsum & !0x000000ff;
        let replaced = blanked | (ttl as u32);
        self.rest = self.rest.replace(1, replaced);
    }

    #[inline]
    pub fn protocol(&self) -> u8 {
        let ttlpcsum = self.rest.extract(1);
        ((ttlpcsum & 0xff00) >> 8) as u8
    }

    #[inline]
    pub fn set_protocol(&mut self, protocol: u8) {
        let ttlpcsum = self.rest.extract(1);
        let blanked = ttlpcsum & !0xff00;
        let replaced = blanked | ((protocol as u32) << 8);
        self.rest = self.rest.replace(1, replaced);
    }

    #[inline]
    pub fn csum(&self) -> u16 {
        let ttlpcsum = self.rest.extract(1);
        ((ttlpcsum & 0xffff0000) >> 16) as u16
    }

    #[inline]
    pub fn set_csum(&mut self, csum: u16) {
        let ttlpcsum = self.rest.extract(1);
        let blanked = ttlpcsum & !0xffff0000;
        let replaced = blanked | ((u16::to_be(csum) as u32) << 16);
        self.rest = self.rest.replace(1, replaced);
    }

    #[inline]
    pub fn id(&self) -> u16 {
        let id_flag_fragment = self.rest.extract(0);
        u16::from_be((id_flag_fragment & 0xffff) as u16)
    }

    #[inline]
    pub fn set_id(&mut self, id: u16) {
        let id_flag_fragment = self.rest.extract(0);
        let blanked = id_flag_fragment & !0xffff;
        let replaced = blanked | (u16::to_be(id) as u32);
        self.rest = self.rest.replace(0, replaced);
    }

    #[inline]
    pub fn flags(&self) -> u8 {
        let id_flag_fragment = self.rest.extract(0);
        let flag_fragment = (id_flag_fragment >> 16) as u16;
        (flag_fragment & 0x7) as u8
    }

    #[inline]
    pub fn set_flags(&mut self, flags: u8) {
        let flags_correct = (flags & 0x7) as u32; // Remove any extra bits because that would suck.
        let flags_mask = !0x70000;
        let id_flag_fragment = self.rest.extract(0);
        let blanked = id_flag_fragment & flags_mask;
        let replaced = blanked | (flags_correct << 16);
        self.rest = self.rest.replace(0, replaced);
    }

    #[inline]
    pub fn fragment_offset(&self) -> u16 {
        let id_flag_fragment = self.rest.extract(0);
        let flag_fragment = (id_flag_fragment & 0xffff) as u16;
        u16::from_be(((flag_fragment & !0xe) >> 3))
    }

    #[inline]
    pub fn set_fragment_offset(&mut self, offset: u16) {
        let offset_correct = u16::to_be(offset & 0x1fff) as u32;
        let offset_shifted = offset_correct << 19;
        let blanked = self.rest.extract(0) & 0xfff80000;
        let replaced = blanked | offset_shifted;
        self.rest = self.rest.replace(0, replaced);
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

    #[inline]
    pub fn apply(&mut self, from: &Self) {
        self.version_to_len = from.version_to_len;
        self.rest = from.rest;
    }
}
