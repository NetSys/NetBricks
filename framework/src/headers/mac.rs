use super::{EndOffset, HeaderUpdates};
use common::*;
use headers::{NextHeader, NullHeader};
use hex;
use num::FromPrimitive;
use std::default::Default;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
use std::str::FromStr;

const IPV4_ETHER_TYPE: u16 = 0x0800;
const IPV6_ETHER_TYPE: u16 = 0x86DD;
const VLAN_TAG_FRAME: u16 = 0x8100;
const VLAN_TAG_FRAME_DBL: u16 = 0x9100;

const HDR_SIZE: usize = 14;
const HDR_SIZE_802_1Q: usize = HDR_SIZE + 4;
const HDR_SIZE_802_1AD: usize = HDR_SIZE_802_1Q + 4;

#[derive(FromPrimitive, Debug, PartialEq)]
#[repr(u16)]
pub enum EtherType {
    IPv4 = IPV4_ETHER_TYPE,
    IPv6 = IPV6_ETHER_TYPE,
    VlanTF = VLAN_TAG_FRAME,
    VlanTFDbl = VLAN_TAG_FRAME_DBL,
}

#[derive(Default, Debug)]
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

impl FromStr for MacAddress {
    type Err = Error;
    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        match hex::decode(s.replace(":", "").replace("-", "")) {
            Ok(ref v) if v.len() == 6 => Ok(MacAddress::new_from_slice(v.as_slice())),
            _ => Err(Error::from_kind(ErrorKind::FailedToParseMacAddress(
                s.to_string(),
            ))),
        }
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
#[derive(Default)]
#[repr(C, packed)]
pub struct MacHeader {
    pub dst: MacAddress,
    pub src: MacAddress,
    pub etype: u16,
}

impl fmt::Display for MacHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} > {} 0x{:04x}", self.src, self.dst, self.etype)
    }
}

impl EndOffset for MacHeader {
    type PreviousHeader = NullHeader;
    #[inline]
    fn offset(&self) -> usize {
        if cfg!(feature = "performance") {
            HDR_SIZE
        } else {
            match self.etype {
                VLAN_TAG_FRAME => HDR_SIZE_802_1Q,
                VLAN_TAG_FRAME_DBL => HDR_SIZE_802_1AD,
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
    pub fn etype(&self) -> Option<EtherType> {
        FromPrimitive::from_u16(u16::from_be(self.etype))
    }

    #[inline]
    pub fn set_etype(&mut self, etype: EtherType) {
        self.etype = u16::to_be(etype as u16)
    }

    #[inline]
    pub fn swap_addresses(&mut self) {
        let mut src: MacAddress = Default::default();
        src.copy_address(&self.src);
        self.src.copy_address(&self.dst);
        self.dst.copy_address(&src);
    }
}

impl HeaderUpdates for MacHeader {
    type PreviousHeader = NullHeader;

    #[inline]
    fn update_payload_len(&mut self, _payload_diff: isize) {}

    #[inline]
    fn update_next_header(&mut self, _hdr: NextHeader) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        let address = "ba:dc:af:eb:ee:f4";
        let expected = address.to_string();
        let parsed = MacAddress::from_str(address).map(|a| a.to_string());
        assert_eq!(parsed.ok(), Some(expected));
    }

    #[test]
    fn test_from_str_dashes() {
        let address = "ba-dc-af-eb-ee-f2";
        let expected = "ba:dc:af:eb:ee:f2".to_string();
        let parsed = MacAddress::from_str(address).map(|a| a.to_string());
        assert_eq!(parsed.ok(), Some(expected));
    }

    #[test]
    fn test_from_str_no_delimiters() {
        let address = "badcafebeef3";
        let expected = "ba:dc:af:eb:ee:f3".to_string();
        let parsed = MacAddress::from_str(address).map(|a| a.to_string());
        assert_eq!(parsed.ok(), Some(expected));
    }

    #[test]
    fn test_unparseable_address() {
        let address = "go:od:ca:fe:be:ef";
        let parsed = MacAddress::from_str(address);
        assert!(parsed.is_err());
        match parsed.err() {
            Some(Error(ErrorKind::FailedToParseMacAddress(s), _)) => {
                assert_eq!(address.to_string(), s);
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_unparseable_address2() {
        let address = "ab:ad:ad:d4";
        let parsed = MacAddress::from_str(address);
        assert!(parsed.is_err());
        match parsed.err() {
            Some(Error(ErrorKind::FailedToParseMacAddress(s), _)) => {
                assert_eq!(address.to_string(), s);
            }
            _ => assert!(false),
        }
    }
}
