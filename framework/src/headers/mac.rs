use super::EndOffset;
use headers::NullHeader;
use std::default::Default;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;

#[derive(Debug, Default)]
#[repr(C, packed)]
pub struct MacAddress {
    pub addr: [u8; 6],
}

impl fmt::Display for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.addr[0],
            self.addr[1],
            self.addr[2],
            self.addr[3],
            self.addr[4],
            self.addr[5]
        )
    }
}

impl MacAddress {
    pub fn new(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8) -> MacAddress {
        MacAddress { addr: [a, b, c, d, e, f] }
    }

    pub fn new_from_slice(slice: &[u8]) -> MacAddress {
        MacAddress { addr: [slice[0], slice[1], slice[2], slice[3], slice[4], slice[5]] }
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

/// A packet's MAC header, with up to two VLAN tags.
#[derive(Debug, Default)]
#[repr(C, packed)]
pub struct MacHeader {
    pub dst: MacAddress,
    pub src: MacAddress,
    etype0: u16, // 0x8100 (8021.Q), 0x88a8 (QinQ), or ethertype of the next header
    tci1: u16,
    etype1: u16, // if valid, 0x8100 (8021.Q) or ethertype of the next header
    tci2: u16,
    etype2: u16, // if valid, ethertype of next header
}

impl fmt::Display for MacHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut result: fmt::Result;
        result =
            write!(f,
               "{} > {} etype=0x{:04x}",
               self.src,
               self.dst,
               self.etype(),
        );
        if self.num_tags() >= 1 {
            result = write!(f, " vid1=0x{:03x}", self.vid1());
        }

        if self.num_tags() >= 2 {
            result = write!(f, " vid2=0x{:03x}", self.vid2())
        }
        return result;
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
            match self.etype0 {
                0x8100 => HDR_SIZE_802_1Q,
                0x9100 => HDR_SIZE_802_1AD,
                _ => HDR_SIZE,
            }
        }
    }
    #[inline]
    fn size() -> usize {
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

    // Return the ethernet type of the next header.
    #[inline]
    pub fn etype(&self) -> u16 {
        match u16::from_be(self.etype0) {
            0x8100 => u16::from_be(self.etype1),
            0x88a8 => u16::from_be(self.etype2),
            _ => u16::from_be(self.etype0),
        }
    }

    // Set the ethernet type of the next header.
    #[inline]
    pub fn set_etype(&mut self, etype: u16) {
        match u16::from_be(self.etype0) {
            0x8100 => self.etype1 = u16::to_be(etype),
            0x88a8 => self.etype2 = u16::to_be(etype),
            _ => self.etype0 = u16::to_be(etype),
        }
    }

    #[inline]
    pub fn swap_addresses(&mut self) {
        let mut src: MacAddress = Default::default();
        src.copy_address(&self.src);
        self.src.copy_address(&self.dst);
        self.dst.copy_address(&src);
    }

    // Return the number of VLAN tags.
    #[inline]
    pub fn num_tags(&self) -> u32 {
        match u16::from_be(self.etype0) {
            0x8100 => 1,
            0x88a8 => 2,
            _ => 0,
        }
    }

    // Return the first VLAN tag.
    #[inline]
    pub fn vid1(&self) -> u16 {
        assert!(self.num_tags() >= 1);
        return u16::from_be(self.tci1) & 0x0FFF;
    }

    // Return the second VLAN tag.
    #[inline]
    pub fn vid2(&self) -> u16 {
        assert!(self.num_tags() >= 2);
        return u16::from_be(self.tci2) & 0x0FFF;
    }

    // Set the first VLAN tag.
    #[inline]
    pub fn set_vid1(&mut self, vid: u16) {
        assert!(self.num_tags() >= 1);
        assert!(vid <= 0xFFF);
        let pri = (self.tci1 & 0x0F) << 4;
        self.tci1 = pri | u16::to_be(vid);
    }

    // Set the second VLAN tag
    #[inline]
    pub fn set_vid2(&mut self, vid: u16) {
        assert!(self.num_tags() >= 1);
        assert!(vid <= 0xFFF);
        let pri = (self.tci2 & 0x0F) << 4;
        self.tci2 = pri | u16::to_be(vid);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_1tagged(pri: u16, vid: u16) -> MacHeader {
        assert!(pri <= 0xF, "test config failure");
        assert!(vid <= 0xFFF, "test config failure");
        let tci = u16::to_be((pri << 12) | vid);
        MacHeader {
            dst: MacAddress { addr: [0, 1, 2, 3, 4, 5] },
            src: MacAddress { addr: [5, 4, 3, 2, 1, 0] },
            etype0: u16::to_be(0x8100),
            tci1: tci,
            etype1: u16::to_be(0x8000), // IPv4
            tci2: 0,
            etype2: 0,
        }
    }

    #[test]
    fn test_1tagged() {
        let pairs: Vec<(u16, u16)> = vec![(3, 13), (6, 100), (0, 0), (0, 0xFFF), (0xF, 0xFFF)];
        let mut new_vid: u16 = 0x00E;
        for &(pri, vid) in &pairs {
            let mut mac = build_1tagged(pri, vid);
            assert_eq!(1, mac.num_tags());
            assert_eq!(vid, mac.vid1());
            assert_eq!(0x8000, mac.etype());

            mac.set_vid1(new_vid);
            assert_eq!(new_vid, mac.vid1());
            assert_eq!(0x8000, mac.etype());

            mac.set_etype(0x1131);
            assert_eq!(new_vid, mac.vid1());
            assert_eq!(0x1131, mac.etype());
            assert_eq!(0x1131, u16::from_be(mac.etype1));
            assert_eq!(1, mac.num_tags());

            new_vid += 0x031;
        }
    }

    fn build_2tagged(pri: u16, vid: u16, pri2: u16, vid2: u16) -> MacHeader {
        assert!(pri <= 0xF, "test config failure");
        assert!(vid <= 0xFFF, "test config failure");
        assert!(pri2 <= 0xF, "test config failure");
        assert!(vid2 <= 0xFFF, "test config failure");
        let tci = u16::to_be((pri << 12) | vid);
        let tci2 = u16::to_be((pri2 << 12) | vid2);
        MacHeader {
            dst: MacAddress { addr: [0, 1, 2, 3, 4, 5] },
            src: MacAddress { addr: [5, 4, 3, 2, 1, 0] },
            etype0: u16::to_be(0x88a8),
            tci1: tci,
            etype1: u16::to_be(0x8100),
            tci2: tci2,
            etype2: u16::to_be(0x8000), // IPv4
        }
    }

    #[test]
    fn test_2tagged() {
        let quads: Vec<(u16, u16, u16, u16)> = vec![
            (3, 13, 5, 16),
            (6, 100, 3, 13),
            (0, 0, 1, 1),
            (0, 0xFFF, 3, 0xFFF),
            (0xF, 0xFFF, 0xF, 0xFFF),
        ];
        let mut new_vid: u16 = 0x00E;
        for &(pri, vid, pri2, vid2) in &quads {
            let mut mac = build_2tagged(pri, vid, pri2, vid2);
            assert_eq!(2, mac.num_tags());
            assert_eq!(vid, mac.vid1());
            assert_eq!(vid2, mac.vid2());
            assert_eq!(0x8000, mac.etype());

            mac.set_vid1(new_vid);
            mac.set_vid2(new_vid + 3);
            assert_eq!(new_vid, mac.vid1());
            assert_eq!(new_vid + 3, mac.vid2());
            assert_eq!(0x8000, mac.etype());

            mac.set_etype(0x1131);
            assert_eq!(new_vid, mac.vid1());
            assert_eq!(0x1131, mac.etype());
            assert_eq!(0x1131, u16::from_be(mac.etype2));
            assert_eq!(2, mac.num_tags());

            new_vid += 0x031;
        }
    }
}
