use serde::{de, Deserialize, Deserializer};
use std::fmt;
use std::net::{IpAddr, Ipv6Addr};
use std::str::FromStr;
use utils::cidr::{Cidr, CidrParseError};

const IPV6ADDR_BITS: usize = 128;

#[derive(Clone, Debug, PartialEq)]
pub struct Ipv6Cidr {
    address: Ipv6Addr,
    length: usize,
    prefix: u128,
    mask: u128,
}

impl Default for Ipv6Cidr {
    fn default() -> Ipv6Cidr {
        Ipv6Cidr {
            address: Ipv6Addr::UNSPECIFIED,
            length: Default::default(),
            prefix: Default::default(),
            mask: Default::default(),
        }
    }
}

impl Cidr for Ipv6Cidr {
    type Addr = Ipv6Addr;

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
            0 => u128::max_value(),
            1..=IPV6ADDR_BITS => u128::max_value() << (IPV6ADDR_BITS - length),
            _ => return Err(CidrParseError("Not a valid length".to_string())),
        };

        let prefix = u128::from_be_bytes(address.octets()) & mask;

        Ok(Ipv6Cidr {
            address,
            length,
            prefix,
            mask,
        })
    }

    #[inline]
    fn contains(&self, address: Self::Addr) -> bool {
        self.prefix == (self.mask & u128::from_be_bytes(address.octets()))
    }

    #[inline]
    fn contains_ip(&self, ip: IpAddr) -> bool {
        match ip {
            IpAddr::V6(ip) => self.contains(ip),
            _ => false,
        }
    }
}

impl FromStr for Ipv6Cidr {
    type Err = CidrParseError;

    fn from_str(s: &str) -> Result<Self, CidrParseError> {
        match s.split('/').collect::<Vec<&str>>().as_slice() {
            [addr, len] => {
                let address =
                    Ipv6Addr::from_str(addr).map_err(|e| CidrParseError(e.to_string()))?;
                let length =
                    usize::from_str_radix(len, 10).map_err(|e| CidrParseError(e.to_string()))?;

                Ipv6Cidr::new(address, length)
            }
            _ => Err(CidrParseError("No `/` found".to_string())),
        }
    }
}

impl<'de> Deserialize<'de> for Ipv6Cidr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = <String>::deserialize(deserializer)?;
        Ipv6Cidr::from_str(&s).map_err(de::Error::custom)
    }
}

impl fmt::Display for Ipv6Cidr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.address(), self.length())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use failure::Fail;

    const IPV6_SEGMENT: &str = "[0-9a-f]{1,4}";

    #[test]
    fn parse_bad_cidr() {
        assert!(Ipv6Cidr::from_str("not-a-cidr").is_err());
    }

    #[test]
    fn get_cidr_parse_error() {
        let bad = Ipv6Cidr::new(Ipv6Addr::from_str("acdc::1").unwrap(), 129);
        assert!(bad.is_err());
        let e = bad.unwrap_err();
        let err = format_err!("Failed to parse CIDR: {}", "Not a valid length");
        assert_eq!(e.to_string(), err.to_string());
        assert_eq!(e.name(), Some("netbricks::utils::cidr::CidrParseError"))
    }

    proptest! {
        #[test]
        fn parse_cidr(
            a in IPV6_SEGMENT, b in IPV6_SEGMENT, c in IPV6_SEGMENT, d in IPV6_SEGMENT,
            e in IPV6_SEGMENT, f in IPV6_SEGMENT, g in IPV6_SEGMENT, h in IPV6_SEGMENT, length in 0..128
        ) {
            let cidr = format!("{}:{}:{}:{}:{}:{}:{}:{}/{}", a, b, c, d, e, f, g, h, length);
            assert!(Ipv6Cidr::from_str(cidr.as_str()).is_ok());
        }
    }

    #[test]
    fn cidr_contains_address() {
        let cidr = Ipv6Cidr::from_str("acdc::0/16").unwrap();

        assert!(cidr.contains(Ipv6Addr::from_str("acdc::1").unwrap()));
        assert!(!cidr.contains(Ipv6Addr::from_str("acdb::1").unwrap()));
    }
}
