use super::EndOffset;
use headers::NullHeader;
use std::default::Default;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;

#[derive(Debug, Copy, Default)]
#[repr(C, packed)]
pub struct MacAddress {
    pub addr: [u8; 6],
}

impl fmt::Display for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.addr[0], self.addr[1], self.addr[2], self.addr[3], self.addr[4], self.addr[5]
        )
    }
}

impl MacAddress {
    pub fn new(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8) -> MacAddress {
        MacAddress {
            addr: [a, b, c, d, e, f],
        }
    }

    pub fn new_from_slice(slice: &[u8]) -> MacAddress {
        MacAddress {
            addr: [slice[0], slice[1], slice[2], slice[3], slice[4], slice[5]],
        }
    }

    #[inline]
    pub fn copy_address(&mut self, other: &MacAddress) {
        self.addr.copy_from_slice(&other.addr);
    }
}

impl Clone for MacAddress {
    fn clone(&self) -> MacAddress {
        let mut m: MacAddress = Default::default();
        m.addr.copy_from_slice(&self.addr);
        m
    }
    fn clone_from(&mut self, source: &MacAddress) {
        self.addr.copy_from_slice(&source.addr)
    }
}

impl PartialEq for MacAddress {
    fn eq(&self, other: &MacAddress) -> bool {
        self.addr == other.addr
    }
}

impl Eq for MacAddress {}

impl Hash for MacAddress {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.addr.hash(state);
    }
}

/// A packet's MAC header.
#[derive(Debug, Clone, Copy, Default)]
#[repr(C, packed)]
pub struct MacHeader {
    pub dst: MacAddress,
    pub src: MacAddress,
    etype: u16,
}

impl fmt::Display for MacHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} > {} 0x{:04x}", self.src, self.dst, u16::from_be(self.etype))
    }
}

const HDR_SIZE: usize = 14;
const HDR_SIZE_802_1Q: usize = HDR_SIZE + 4;
const HDR_SIZE_802_1AD: usize = HDR_SIZE_802_1Q + 4;

impl EndOffset for MacHeader {
    type PreviousHeader = NullHeader;
    #[inline]
    fn offset(&self) -> usize {
        if cfg!(feature = "performance") {
            HDR_SIZE
        } else {
            match self.etype {
                0x8100 => HDR_SIZE_802_1Q,
                0x9100 => HDR_SIZE_802_1AD,
                _ => HDR_SIZE,
            }
        }
    }
    #[inline]
    fn size() -> usize {
        // The struct itself is always 20 bytes.
        HDR_SIZE
    }

    #[inline]
    fn payload_size(&self, hint: usize) -> usize {
        hint - self.offset()
    }

    #[inline]
    fn check_correct(&self, _: &NullHeader) -> bool {
        true
    }
}

impl MacHeader {
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn etype(&self) -> u16 {
        u16::from_be(self.etype)
    }

    #[inline]
    pub fn set_etype(&mut self, etype: u16) {
        self.etype = u16::to_be(etype)
    }

    #[inline]
    pub fn swap_addresses(&mut self) {
        let mut src: MacAddress = Default::default();
        src.copy_address(&self.src);
        self.src.copy_address(&self.dst);
        self.dst.copy_address(&src);
    }
}
