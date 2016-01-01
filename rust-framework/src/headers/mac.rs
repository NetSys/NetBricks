use super::super::io;
use std::fmt;

/// A packet's MAC header.
#[derive(Debug)]
#[repr(C)]
pub struct MacHeader {
    pub dst: [u8; 6],
    pub src: [u8; 6],
    pub etype: u16,
}

impl fmt::Display for MacHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x} > {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x} 0x{:04x}",
               self.src[0], self.src[1], self.src[2], self.src[3], self.src[4], self.src[5],
               self.dst[0], self.dst[1], self.dst[2], self.dst[3], self.dst[4], self.dst[5],
               u16::from_be(self.etype))
    }
}

const HDR_SIZE: usize = 14;
const HDR_SIZE_802_1Q: usize = HDR_SIZE + 4;
const HDR_SIZE_802_1AD: usize = HDR_SIZE_802_1Q + 4;

impl io::EndOffset for MacHeader {
    #[inline]
    fn offset(&self) -> usize {
        match self.etype {
            0x8100 => HDR_SIZE_802_1Q,
            0x9100 => HDR_SIZE_802_1AD,
            _ => HDR_SIZE,
        }
    }
}
