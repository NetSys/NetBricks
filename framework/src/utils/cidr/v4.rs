use serde::{de, Deserialize, Deserializer};
use std::fmt;
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use utils::cidr::{Cidr, CidrParseError};

const IPV4ADDR_BITS: usize = 32;

#[derive(Debug, Clone)]
pub struct Ipv4Cidr {
    address: Ipv4Addr,
    length: usize,
    prefix: u32,
    mask: u32,
}

impl Cidr for Ipv4Cidr {
    type Addr = Ipv4Addr;

    #[inline]
    fn address(&self) -> Self::Addr {
        self.address
    }

    #[inline]
    fn length(&self) -> usize {
        self.length
    }

    #[inline]
    fn new(address: Self::Addr, length: usize) -> Result<Self, CidrParseError> {
        let mask = match length {
            0 => u32::max_value(),
            1..=IPV4ADDR_BITS => u32::max_value() << (IPV4ADDR_BITS - length),
            _ => return Err(CidrParseError("Not a valid length".to_string())),
        };

        let prefix = u32::from_be_bytes(address.octets()) & mask;

        Ok(Ipv4Cidr {
            address,
            length,
            prefix,
            mask,
        })
    }

    #[inline]
    fn contains(&self, address: Self::Addr) -> bool {
        self.prefix == (self.mask & u32::from_be_bytes(address.octets()))
    }

    #[inline]
    fn contains_ip(&self, ip: IpAddr) -> bool {
        match ip {
            IpAddr::V4(ip) => self.contains(ip),
            _ => false,
        }
    }
}

impl FromStr for Ipv4Cidr {
    type Err = CidrParseError;

    fn from_str(s: &str) -> Result<Self, CidrParseError> {
        match s.split('/').collect::<Vec<&str>>().as_slice() {
            [addr, len] => {
                let address =
                    Ipv4Addr::from_str(addr).map_err(|e| CidrParseError(e.to_string()))?;
                let length =
                    usize::from_str_radix(len, 10).map_err(|e| CidrParseError(e.to_string()))?;

                Ipv4Cidr::new(address, length)
            }
            _ => Err(CidrParseError("No `/` found".to_string())),
        }
    }
}

impl<'de> Deserialize<'de> for Ipv4Cidr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = <String>::deserialize(deserializer)?;
        Ipv4Cidr::from_str(&s).map_err(de::Error::custom)
    }
}

impl fmt::Display for Ipv4Cidr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.address(), self.length())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use failure::Fail;

    const IPV4_SEGMENT: ::std::ops::Range<u32> = 0..255u32;

    #[test]
    fn parse_bad_cidr() {
        assert!(Ipv4Cidr::from_str("not-a-cidr").is_err());
    }

    #[test]
    fn get_cidr_parse_error() {
        let bad = Ipv4Cidr::new(Ipv4Addr::from_str("10.0.0.0").unwrap(), 99);
        assert!(bad.is_err());
        let e = bad.unwrap_err();
        let err = format_err!("Failed to parse CIDR: {}", "Not a valid length");
        assert_eq!(e.to_string(), err.to_string());
        assert_eq!(e.name(), Some("netbricks::utils::cidr::CidrParseError"))
    }

    proptest! {
        #[test]
        fn parse_cidr(
            a in IPV4_SEGMENT, b in IPV4_SEGMENT, c in IPV4_SEGMENT, d in IPV4_SEGMENT, length in 0..32
        ) {
            let cidr = format!("{}.{}.{}.{}/{}", a, b, c, d, length);
            assert!(Ipv4Cidr::from_str(cidr.as_str()).is_ok());
        }
    }

    #[test]
    fn cidr_contains_address() {
        let cidr = Ipv4Cidr::from_str("10.0.0.0/25").unwrap();
        assert!(cidr.contains(Ipv4Addr::from_str("10.0.0.0").unwrap()));
        assert!(cidr.contains(Ipv4Addr::from_str("10.0.0.127").unwrap()));
        assert!(!cidr.contains(Ipv4Addr::from_str("10.0.0.128").unwrap()));
    }
}
