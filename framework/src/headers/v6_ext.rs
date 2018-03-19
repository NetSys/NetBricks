use super::EndOffset;
use headers::ip::{Ipv6Header};
use std::convert::From;
use std::default::Default;
use std::fmt;
use std::net::{Ipv6Addr};
use std::slice;
use core::mem::size_of;

pub trait Ipv6VarHeader: EndOffset + Default {
    fn next_header(&self) -> u8;
}

impl Ipv6VarHeader for Ipv6Header {
    fn next_header(&self) -> u8 {
        self.next_header
    }
}

#[derive(Default)]
#[repr(C, packed)]
struct Ipv6ExtHeader {
    next_header: u8,
    hdr_ext_len: u8
}

impl Ipv6VarHeader for Ipv6ExtHeader {
    fn next_header(&self) -> u8 {
        self.next_header
    }
}

#[derive(Default)]
#[repr(C, packed)]
struct RoutingHeader {
    routing_type: u8,
    segments_left: u8
}

// The segment list is of variable length
type Segments = &[u128];

#[derive(Default)]
#[repr(C, packed)]
struct SegmentRoutingHeader {
    ext_header: Ipv6ExtHeader,
    routing_header: RoutingHeader,
    last_entry: u8,
    flags: u8,
    tag: u16,
    segments: Segments;
}

impl Ipv6VarHeader for SegmentRoutingHeader {
    fn next_header(&self) -> u8 {
        self.ext_header.next_header()
    }
}

impl EndOffset for Ipv6ExtHeader {
    type PreviousHeader = Ipv6VarHeader;

    #[inline]
    fn offset(&self) -> usize {
        (self.hdr_ext_len as usize) * 8 + 8
    }

    #[inline]
    fn size() -> usize {
        2
    }

    #[inline]
    fn payload_size(&self, hint: usize) -> usize {
        hint - (self.offset())
    }

    #[inline]
    fn check_correct(&self, _prev: &Ipv6VarHeader) -> bool {
        true
    }
}

impl EndOffset for SegmentRoutingHeader {
    type PreviousHeader = Ipv6VarHeader;

    #[inline]
    fn offset(&self) -> usize {
        self.ext_header.offset()
    }

    #[inline]
    fn size() -> usize {
        (8 as usize) + (size_of<Segments>())
    }

    #[inline]
    fn payload_size(&self, hint: usize) -> usize {
        hint - self.offset()
    }

    #[inline]
    fn check_correct(&self, _prev: &Ipv6VarHeader) -> bool {
        // self.routing_header.routing_type == 4
        true
    }
}

impl SegmentRoutingHeader {
    fn new() -> SegmentRoutingHeader {
        Default::default()
    }

    fn hdr_ext_len(&self) -> u8 {
        self.ext_header.hdr_ext_len
    }

    fn routing_type(&self) -> u8 {
        self.routing_header.routing_type
    }

    fn set_routing_type(&mut self, routing_type: u8) {
        self.routing_header.routing_type = routing_type;
    }

    fn segments_left(&self) -> u8 {
        self.routing_header.segments_left
    }

    fn set_segments_left(&mut self, segments_left: u8) {
        self.routing_header.segments_left = segments_left;
    }

    fn last_entry(&self) -> u8 {
        self.last_entry
    }

    fn set_last_entry(&mut self, last_entry: u8) {
        self.last_entry = last_entry;
    }

    // flags
    fn protected(&self) -> bool {
        (self.flags & 0x40) > 0
    }

    fn set_protected(&mut self, protected: bool) {
        let bit: u8 = if protected { 0x40 } else { 0 };
        self.flags = (self.flags & !0x40) | bit;
    }

    fn oam(&self) -> bool {
        (self.flags & 0x20) > 0
    }

    fn set_oam(&mut self, oam: bool) {
        let bit: u8 = if oam { 0x20 } else { 0 };
        self.flags = (self.flags & !0x20) | bit;
    }

    fn alert(&self) -> bool {
        (self.flags & 0x10) > 0
    }

    fn set_alert(&mut self, alert: bool) {
        let bit: u8 = if alert { 0x10 } else { 0 };
        self.flags = (self.flags & !0x10) | bit;
    }

    fn hmac(&self) -> bool {
        (self.flags & 0x08) > 0
    }

    fn set_hmac(&mut self, hmac: bool) {
        let bit: u8 = if hmac { 0x08 } else { 0 };
        self.flags = (self.flags & !0x08) | bit;
    }

    fn segments(&self) -> Segments {
        let ptr = &self.segments as const *u128;
        let len = self.ext_header.offset() - 8;
        unsafe {
            slice::from_raw_parts(ptr, len)
        }
    }
}
