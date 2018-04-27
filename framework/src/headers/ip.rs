use super::EndOffset;
use byteorder::{BigEndian, ByteOrder};
use headers::MacHeader;
use std::convert::From;
use std::default::Default;
use std::fmt;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::slice;
use utils::{Flow, FlowV6};

pub type Ipv4Address = u32;
pub type Ipv6Address = u128;

/// IP header using SSE
#[derive(Default)]
#[repr(C, packed)]
pub struct Ipv4Header {
    version_to_len: u32,
    id_to_foffset: u32,
    ttl_to_csum: u32,
    src_ip: Ipv4Address,
    dst_ip: Ipv4Address,
}

#[derive(Default)]
#[repr(C, packed)]
pub struct Ipv6Header {
    version_to_flow_label: u32,
    payload_len: u16,
    next_header: u8,
    hop_limit: u8,
    src_ip: Ipv6Address,
    dst_ip: Ipv6Address,
}

pub trait IpHeader: EndOffset + Default {}

impl IpHeader for Ipv4Header {}
impl IpHeader for Ipv6Header {}

impl fmt::Display for Ipv4Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let src = Ipv4Addr::from(self.src());
        let dst = Ipv4Addr::from(self.dst());
        write!(
            f,
            "{} > {} version: {} ihl: {} len: {} ttl: {} proto: {} csum: {}",
            src,
            dst,
            self.version(),
            self.ihl(),
            self.length(),
            self.ttl(),
            self.protocol(),
            self.csum()
        )
    }
}

impl fmt::Display for Ipv6Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let src = Ipv6Addr::from(self.src());
        let dst = Ipv6Addr::from(self.dst());
        write!(
            f,
            "{} > {} version: {} traffic_class: {} flow_label: {} len: {} next_header: {} hop_limit: {}",
            src,
            dst,
            self.version(),
            self.traffic_class(),
            self.flow_label(),
            self.payload_len(),
            self.next_header(),
            self.hop_limit()
        )
    }
}

impl EndOffset for Ipv4Header {
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

impl EndOffset for Ipv6Header {
    type PreviousHeader = MacHeader;

    #[inline]
    fn offset(&self) -> usize {
        // IPv6 Header is always 40 bytes: (4 + 8 + 20 + 16 + 8 + 8 + 128 + 128) / 8 = 40
        40
    }

    #[inline]
    fn size() -> usize {
        // Struct is always 40 bytes as well
        40
    }

    #[inline]
    fn payload_size(&self, _: usize) -> usize {
        self.payload_len() as usize
    }

    #[inline]
    fn check_correct(&self, _prev: &MacHeader) -> bool {
        true
    }
}

impl Ipv4Header {
    #[inline]
    pub fn flow(&self) -> Option<Flow> {
        let protocol = self.protocol();
        let src_ip = self.src();
        let dst_ip = self.dst();
        if (protocol == 6 || protocol == 17) && self.payload_size(0) >= 4 {
            unsafe {
                let self_as_u8 = (self as *const Ipv4Header) as *const u8;
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
    pub fn new() -> Ipv4Header {
        Default::default()
    }

    #[inline]
    pub fn src(&self) -> Ipv4Address {
        Ipv4Address::from_be(self.src_ip)
    }

    #[inline]
    pub fn set_src(&mut self, src: Ipv4Address) {
        self.src_ip = Ipv4Address::to_be(src)
    }

    #[inline]
    pub fn dst(&self) -> Ipv4Address {
        Ipv4Address::from_be(self.dst_ip)
    }

    #[inline]
    pub fn set_dst(&mut self, dst: Ipv4Address) {
        self.dst_ip = Ipv4Address::to_be(dst);
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
        self.id_to_foffset =
            (self.id_to_foffset & !0x00e00000) | (((flags & 0x7) as u32) << (16 + 5));
    }

    #[inline]
    pub fn fragment_offset(&self) -> u16 {
        let id_flag_fragment = self.id_to_foffset;
        let flag_fragment = (id_flag_fragment & 0xffff) as u16;
        u16::from_be((flag_fragment & !0xe) >> 3)
    }

    #[inline]
    pub fn set_fragment_offset(&mut self, offset: u16) {
        let offset_correct = offset as u32;
        let id_to_offset_le = u32::from_be(self.id_to_foffset);
        self.id_to_foffset = u32::to_be(id_to_offset_le & !0x1fff | offset_correct);
    }

    #[inline]
    pub fn version(&self) -> u8 {
        // ((self.version_to_len & 0xf0) as u8) >> 4
        ((u32::from_be(self.version_to_len) & 0xf0000000) >> 28) as u8
    }

    #[inline]
    pub fn set_version(&mut self, version: u8) {
        self.version_to_len = u32::to_be(
            (((version as u32) << 28) & 0xf0000000)
                | (u32::from_be(self.version_to_len) & !0xf0000000),
        );
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
        self.version_to_len =
            (self.version_to_len & !0xffff0000) | ((u16::to_be(len) as u32) << 16);
    }
}

impl Ipv6Header {
    #[inline]
    pub fn flow(&self) -> Option<FlowV6> {
        let next_hdr = self.next_header();
        let src_ip = self.src();
        let dst_ip = self.dst();
        if (next_hdr == 6 || next_hdr == 17) && self.payload_size(0) >= 4 {
            unsafe {
                let self_as_u8 = (self as *const Ipv6Header) as *const u8;
                let port_as_u8 = self_as_u8.offset(self.offset() as isize);
                let port_slice = slice::from_raw_parts(port_as_u8, 4);
                let dst_port = BigEndian::read_u16(&port_slice[..2]);
                let src_port = BigEndian::read_u16(&port_slice[2..]);
                Some(FlowV6 {
                    src_ip: src_ip,
                    dst_ip: dst_ip,
                    src_port: src_port,
                    dst_port: dst_port,
                    proto: next_hdr,
                })
            }
        } else {
            None
        }
    }

    #[inline]
    pub fn new() -> Ipv6Header {
        Default::default()
    }

    #[inline]
    pub fn src(&self) -> Ipv6Address {
        Ipv6Address::from_be(self.src_ip)
    }

    #[inline]
    pub fn set_src(&mut self, src: Ipv6Address) {
        self.src_ip = Ipv6Address::to_be(src)
    }

    #[inline]
    pub fn dst(&self) -> Ipv6Address {
        Ipv6Address::from_be(self.dst_ip)
    }

    #[inline]
    pub fn set_dst(&mut self, dst: Ipv6Address) {
        self.dst_ip = Ipv6Address::to_be(dst);
    }

    #[inline]
    pub fn hlimit(&self) -> u8 {
        self.hop_limit
    }

    #[inline]
    pub fn set_hlimit(&mut self, hlimit: u8) {
        self.hop_limit = hlimit;
    }

    #[inline]
    pub fn version(&self) -> u8 {
        ((u32::from_be(self.version_to_flow_label) & 0xf0000000) >> 28) as u8
    }

    #[inline]
    pub fn set_version(&mut self, version: u8) {
        self.version_to_flow_label = u32::to_be(
            (((version as u32) << 28) & 0xf0000000)
                | (u32::from_be(self.version_to_flow_label) & !0xf0000000),
        );
    }

    #[inline]
    pub fn traffic_class(&self) -> u8 {
        ((u32::from_be(self.version_to_flow_label) >> 20) as u8)
    }

    #[inline]
    pub fn set_traffic_class(&mut self, tclass: u8) {
        self.version_to_flow_label = u32::to_be(
            (u32::from_be(self.version_to_flow_label) & 0xf00fffff) | ((tclass as u32) << 20),
        )
    }

    #[inline]
    pub fn flow_label(&self) -> u32 {
        u32::from_be(self.version_to_flow_label) & 0x0fffff
    }

    #[inline]
    pub fn set_flow_label(&mut self, flow_label: u32) {
        assert!(flow_label <= 0x0fffff);
        self.version_to_flow_label = u32::to_be(
            (u32::from_be(self.version_to_flow_label) & 0xfff00000) | (flow_label & 0x0fffff),
        )
    }

    #[inline]
    pub fn payload_len(&self) -> u16 {
        u16::from_be(self.payload_len)
    }

    #[inline]
    pub fn set_payload_len(&mut self, len: u16) {
        self.payload_len = u16::to_be(len)
    }

    #[inline]
    pub fn next_header(&self) -> u8 {
        self.next_header
    }

    #[inline]
    pub fn set_next_header(&mut self, hdr: u8) {
        self.next_header = hdr
    }

    #[inline]
    pub fn hop_limit(&self) -> u8 {
        self.hop_limit
    }

    #[inline]
    pub fn set_hop_limit(&mut self, limit: u8) {
        self.hop_limit = limit
    }
}

#[cfg(test)]
mod ipv4 {
    use super::*;
    use std::net::Ipv4Addr;
    use std::str::FromStr;

    #[test]
    fn packet() {
        let mut ip = Ipv4Header::new();
        ip.set_src(u32::from(Ipv4Addr::from_str("10.0.0.1").unwrap()));
        ip.set_dst(u32::from(Ipv4Addr::from_str("10.0.0.5").unwrap()));
        ip.set_ttl(128);
        ip.set_version(4);
        ip.set_ihl(5);
        ip.set_length(20);

        assert_eq!(ip.version(), 4);
        assert_eq!(ip.length(), 20);
    }
}

#[cfg(test)]
mod ipv6 {
    use super::*;
    use std::net::Ipv6Addr;
    use std::str::FromStr;

    #[test]
    fn packet() {
        let mut ip = Ipv6Header::new();
        let src = Ipv6Addr::from_str("2001:db8::1").unwrap();
        let dst = Ipv6Addr::from_str("2001:db8::2").unwrap();
        ip.set_src(u128::from(src));
        ip.set_dst(u128::from(dst));
        ip.set_version(6);
        ip.set_traffic_class(17);
        ip.set_flow_label(15000);
        ip.set_payload_len(1000);
        ip.set_next_header(17); // UDP
        ip.set_hop_limit(2);

        assert_eq!(ip.version(), 6);
        assert_eq!(ip.traffic_class(), 17);
        assert_eq!(ip.flow_label(), 15000);
        assert_eq!(ip.payload_len(), 1000);
        assert_eq!(ip.next_header(), 17);
        assert_eq!(ip.hop_limit(), 2);
        assert_eq!("2001:db8::1 > 2001:db8::2 version: 6 traffic_class: 17 flow_label: 15000 len: 1000 next_header: 17 hop_limit: 2", ip.to_string())
    }
}
