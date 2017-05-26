use super::EndOffset;
use byteorder::{BigEndian, ByteOrder};
use headers::MacHeader;
use std::convert::From;
use std::default::Default;
use std::fmt;
use std::net::Ipv4Addr;
use std::slice;
use utils::Flow;

/// IP header using SSE
#[derive(Debug, Default)]
#[repr(C,packed)]
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
        write!(f,
               "{} > {} version: {} ihl: {} len: {} ttl: {} proto: {} csum: {}",
               src,
               dst,
               self.version(),
               self.ihl(),
               self.length(),
               self.ttl(),
               self.protocol(),
               self.csum())
    }
}

impl EndOffset for IpHeader {
    type PreviousHeader = MacHeader;
    #[inline]
    fn offset(&self) -> usize {
        if cfg!(feature = "performance") {
            20
        } else {
            self.ihl() as usize * 4
        }
    }

    #[inline]
    fn size() -> usize {
        // The struct itself is always 20 bytes.
        20
    }

    #[inline]
    fn payload_size(&self, _: usize) -> usize {
        (self.length() as usize) - self.offset()
    }

    #[inline]
    fn check_correct(&self, _prev: &MacHeader) -> bool {
        true
    }
}

impl IpHeader {
    #[inline]
    pub fn flow(&self) -> Option<Flow> {
        let protocol = self.protocol();
        let src_ip = self.src();
        let dst_ip = self.dst();
        if (protocol == 6 || protocol == 17) && self.payload_size(0) >= 4 {
            unsafe {
                let self_as_u8 = (self as *const IpHeader) as *const u8;
                let port_as_u8 = self_as_u8.offset(self.offset() as isize);
                let port_slice = slice::from_raw_parts(port_as_u8, 4);
                let dst_port = BigEndian::read_u16(&port_slice[..2]);
                let src_port = BigEndian::read_u16(&port_slice[2..]);
                Some(Flow {
                         src_ip: src_ip,
                         dst_ip: dst_ip,
                         src_port: src_port,
                         dst_port: dst_port,
                         proto: protocol,
                     })
            }
        } else {
            None
        }
    }

    #[inline]
    pub fn new() -> IpHeader {
        Default::default()
    }

    #[inline]
    pub fn src(&self) -> u32 {
        u32::from_be(self.src_ip)
    }

    #[inline]
    pub fn set_src(&mut self, src: u32) {
        self.src_ip = u32::to_be(src)
    }

    #[inline]
    pub fn dst(&self) -> u32 {
        u32::from_be(self.dst_ip)
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
        self.id_to_foffset = (self.id_to_foffset & !0x00e00000) | (((flags & 0x7) as u32) << (16 + 5));
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
        let id_to_offset_le = u32::from_be(self.id_to_foffset);
        self.id_to_foffset = u32::to_be(id_to_offset_le & !0x1fff | offset_correct);
    }

    #[inline]
    pub fn version(&self) -> u8 {
        ((self.version_to_len & 0xf0) as u8) >> 4
    }

    #[inline]
    pub fn set_version(&mut self, version: u8) {
        self.version_to_len = (self.version_to_len & !0xf0) | (((version & 0xf) as u32) << 4);
    }

    #[inline]
    pub fn ihl(&self) -> u8 {
        (self.version_to_len & 0x0f) as u8
    }

    #[inline]
    pub fn set_ihl(&mut self, ihl: u8) {
        self.version_to_len = (self.version_to_len & !0x0f) | ((ihl & 0x0f) as u32);
    }

    #[inline]
    pub fn dscp(&self) -> u8 {
        ((self.version_to_len & 0xfc00) >> 10) as u8
    }

    #[inline]
    pub fn set_dscp(&mut self, dscp: u8) {
        self.version_to_len = (self.version_to_len & !0xfc00) | (((dscp & 0x3f) as u32) << 10);
    }

    #[inline]
    pub fn ecn(&self) -> u8 {
        ((self.version_to_len & 0x0300) >> 8) as u8
    }

    #[inline]
    pub fn set_ecn(&mut self, ecn: u8) {
        self.version_to_len = (self.version_to_len & !0x0300) | (((ecn & 0x03) as u32) << 8);
    }

    #[inline]
    pub fn length(&self) -> u16 {
        u16::from_be(((self.version_to_len & 0xffff0000) >> 16) as u16)
    }

    #[inline]
    pub fn set_length(&mut self, len: u16) {
        self.version_to_len = (self.version_to_len & !0xffff0000) | ((u16::to_be(len) as u32) << 16);
    }
}
