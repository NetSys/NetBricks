use super::EndOffset;
use std::fmt;
use std::default::Default;

#[derive(Debug, Default)]
#[repr(C, packed)]
pub struct MacAddress {
    pub addr: [u8; 6],
}

/// A packet's MAC header.
#[derive(Debug, Default)]
#[repr(C, packed)]
pub struct MacHeader {
    pub dst: [u8; 6],
    pub src: [u8; 6],
    pub etype: u16,
}

impl fmt::Display for MacHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x} > \
                {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x} 0x{:04x}",
               self.src[0],
               self.src[1],
               self.src[2],
               self.src[3],
               self.src[4],
               self.src[5],
               self.dst[0],
               self.dst[1],
               self.dst[2],
               self.dst[3],
               self.dst[4],
               self.dst[5],
               u16::from_be(self.etype))
    }
}

const HDR_SIZE: usize = 14;
const HDR_SIZE_802_1Q: usize = HDR_SIZE + 4;
const HDR_SIZE_802_1AD: usize = HDR_SIZE_802_1Q + 4;

impl EndOffset for MacHeader {
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
}

impl MacHeader {
    pub fn new() -> Self {
        Default::default()
    }
}
