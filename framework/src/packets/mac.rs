use std::fmt;

// mac address
#[derive(Default, Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct MacAddr(pub [u8; 6]);

impl MacAddr {
    pub fn new(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8) -> Self {
        MacAddr([a, b, c, d, e, f])
    }

    pub fn new_from_slice(slice: &[u8]) -> Self {
        MacAddr([slice[0], slice[1], slice[2], slice[3], slice[4], slice[5]])
    }
}

impl fmt::Display for MacAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

// ethernet type
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
#[repr(C, packed)]
pub struct EtherType(pub u16);

impl EtherType {
    pub fn new(value: u16) -> Self {
        EtherType(value)
    }
}

#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub mod EtherTypes {
    use super::EtherType;

    // Internet Protocol version 4
    pub const Ipv4: EtherType = EtherType(0x0800);
    // Internet Protocol version 6
    pub const Ipv6: EtherType = EtherType(0x86DD);
}

impl fmt::Display for EtherType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                &EtherTypes::Ipv4 => "Ipv4".to_string(),
                &EtherTypes::Ipv6 => "Ipv6".to_string(),
                _ => format!("0x{:04x}", self.0)
            }
        )
    }
}

// ethernet header
#[derive(Default, Debug)]
#[repr(C, packed)]
struct MacHeader {
    dst: MacAddr,
    src: MacAddr,
    ether_type: EtherType
}

impl MacHeader {
    fn new() -> Self {
        Default::default()
    }
}

impl fmt::Display for MacHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} > {} [{}]", self.src, self.dst, self.ether_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn str_from_mac_addr() {
        assert_eq!(format!("{}", MacAddr::new(0, 0, 0, 0, 0, 0)), "00:00:00:00:00:00");
        assert_eq!(format!("{}", MacAddr::new(255, 255, 255, 255, 255, 255)), "ff:ff:ff:ff:ff:ff");
        assert_eq!(format!("{}", MacAddr::new(0x12, 0x34, 0x56, 0xAB, 0xCD, 0xEF)), "12:34:56:ab:cd:ef");
    }

    #[test]
    fn str_from_ether_type() {
        assert_eq!(format!("{}", EtherTypes::Ipv4), "Ipv4");
        assert_eq!(format!("{}", EtherTypes::Ipv6), "Ipv6");
        assert_eq!(format!("{}", EtherType::new(0)), "0x0000");
    }

    #[test]
    fn str_from_mac_header() {
        assert_eq!(format!("{}", MacHeader::new()), "00:00:00:00:00:00 > 00:00:00:00:00:00 [0x0000]");
    }
}
