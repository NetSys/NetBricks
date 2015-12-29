use super::mbuf;
use super::interface::htons;
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
        write!(f, "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x} {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x} {:#x}",
               self.dst[0], self.dst[1], self.dst[2], self.dst[3], self.dst[4], self.dst[5],
               self.src[0], self.src[1], self.src[2], self.src[3], self.src[4], self.src[5],
               htons(self.etype))
    }
}

impl mbuf::FromMBuf for MacHeader {
    #[inline]
    fn mut_transform(pkt: &mut mbuf::MBuf) -> &mut MacHeader {
        let pdata = pkt.data_address();
        let typecast = pdata as *mut MacHeader;
        unsafe {&mut *typecast}
    }

    #[inline]
    fn const_transform(pkt: &mbuf::MBuf) -> &MacHeader {
        let pdata = pkt.data_address();
        let typecast = pdata as *const MacHeader;
        unsafe {&*typecast}
    }
}

