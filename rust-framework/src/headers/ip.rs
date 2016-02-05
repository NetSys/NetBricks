use super::super::io;
use std::fmt;
use std::net::Ipv4Addr;
use std::convert::From;

/// IP header using SSE
//#[repr(C, packed)]
#[derive(Debug)]
#[repr(simd)]
pub struct IpHeader {
    version_to_len: u32,
    id_to_foffset: u32,
    ttl_to_csum: u32,
    src_ip: u32,
    dst_ip: u32,
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
        IpHeader{version_to_len: 0, id_to_foffset:0, ttl_to_csum:0, src_ip: 0, dst_ip:0}
    }

    #[inline]
    pub fn src(&self) -> u32 {
        let ip = u32::from_be(self.src_ip);
        ip
    }

    #[inline]
    pub fn set_src(&mut self, src: u32) {
        self.src_ip = u32::to_be(src)
    }

    #[inline]
    pub fn dst(&self) -> u32 {
        let ip = u32::from_be(self.dst_ip);
        ip
    }

    #[inline]
    pub fn set_dst(&mut self, dst: u32) {
        self.dst_ip = u32::to_be(dst);
    }

    #[inline]
    pub fn ttl(&self) -> u8 {
        let ttlpcsum = self.ttl_to_csum;
        (ttlpcsum & 0x000000ff) as u8
    }

    #[inline]
    pub fn set_ttl(&mut self, ttl: u8) {
        let ttlpcsum = self.ttl_to_csum;
        let blanked = ttlpcsum & !0x000000ff;
        self.ttl_to_csum = blanked | (ttl as u32);
    }

    #[inline]
    pub fn protocol(&self) -> u8 {
        let ttlpcsum = self.ttl_to_csum;
        ((ttlpcsum & 0xff00) >> 8) as u8
    }

    #[inline]
    pub fn set_protocol(&mut self, protocol: u8) {
        let ttlpcsum = self.ttl_to_csum;
        let blanked = ttlpcsum & !0xff00;
        self.ttl_to_csum = blanked | ((protocol as u32) << 8);
    }

    #[inline]
    pub fn csum(&self) -> u16 {
        let ttlpcsum = self.ttl_to_csum;
        ((ttlpcsum & 0xffff0000) >> 16) as u16
    }

    #[inline]
    pub fn set_csum(&mut self, csum: u16) {
        let ttlpcsum = self.ttl_to_csum;
        let blanked = ttlpcsum & !0xffff0000;
        self.ttl_to_csum = blanked | ((u16::to_be(csum) as u32) << 16);
    }

    #[inline]
    pub fn id(&self) -> u16 {
        let id_flag_fragment = self.id_to_foffset;
        u16::from_be((id_flag_fragment & 0xffff) as u16)
    }

    #[inline]
    pub fn set_id(&mut self, id: u16) {
        let id_flag_fragment = self.id_to_foffset;
        let blanked = id_flag_fragment & !0xffff;
        self.id_to_foffset = blanked | (u16::to_be(id) as u32);
    }

    #[inline]
    pub fn flags(&self) -> u8 {
        let id_flag_fragment = self.id_to_foffset;
        let flag_fragment = (id_flag_fragment >> 21) as u16;
        (flag_fragment & 0x7) as u8
    }

    #[inline]
    pub fn set_flags(&mut self, flags: u8) {
        self.id_to_foffset = (self.id_to_foffset & !0x00e00000) | 
            (((flags & 0x7) as u32) << 16 + 5);
    }

    #[inline]
    pub fn fragment_offset(&self) -> u16 {
        let id_flag_fragment = self.id_to_foffset;
        let flag_fragment = (id_flag_fragment & 0xffff) as u16;
        u16::from_be(((flag_fragment & !0xe) >> 3))
    }

    #[inline]
    pub fn set_fragment_offset(&mut self, offset: u16) {
        let offset_correct = offset as u32;
        self.id_to_foffset = (self.id_to_foffset & !0x001f0000) | 
                                ((offset_correct & 0x1f00) << 11);
        self.id_to_foffset = (self.id_to_foffset & !0xff000000) |
                                ((offset_correct & 0xff) << 24);
    }

    #[inline]
    pub fn version(&self) -> u8 {
        let vihl = ((self.version_to_len & 0xf0)  as u8) >> 4;
        vihl
    }

    #[inline]
    pub fn set_version(&mut self, version: u8) {
        self.version_to_len = (self.version_to_len & !0xf0) | (((version & 0xf0) as u32) << 4);
    }

    #[inline]
    pub fn ihl(&self) -> u8 {
        let ihl = (self.version_to_len & 0xf) as u8;
        ihl
    }

    #[inline]
    pub fn set_ihl(&mut self, ihl: u8) {
        self.version_to_len = (self.version_to_len & !0xf) | ((ihl & 0xf) as u32);
    }

    #[inline]
    pub fn dscp(&self) -> u8 {
        let dscp_ecn = ((self.version_to_len & 0xfc00) >> 10) as u8;
        dscp_ecn
    }

    #[inline]
    pub fn set_dscp(&mut self, dscp: u8) {
        self.version_to_len = (self.version_to_len & !0xfc00) | (((dscp & 0x3f) as u32) << 10);
    }

    #[inline]
    pub fn ecn(&self) -> u8 {
        let ecn = ((self.version_to_len & 0x0300) >> 8) as u8;
        ecn
    }

    #[inline]
    pub fn set_ecn(&mut self, ecn: u8) {
        self.version_to_len = (self.version_to_len & !0x0300) | (((ecn & 0x03) as u32) << 8); 
    }

    #[inline]
    pub fn length(&self) -> u16 {
        ((self.version_to_len & 0xffff0000) >> 16) as u16
    }

    #[inline]
    pub fn set_length(&mut self, len: u16) {
        self.version_to_len = (self.version_to_len & !0xffff0000) | ((u16::to_be(len) as u32) << 16);
    }

    // FIXME: Make sure this uses SIMD
    #[inline]
    pub fn apply(&mut self, from: &Self) {
        self.version_to_len = from.version_to_len;
        self.id_to_foffset = from.id_to_foffset;
        self.ttl_to_csum = from.ttl_to_csum;
        self.src_ip = from.src_ip;
        self.dst_ip = from.dst_ip;
    }
}
