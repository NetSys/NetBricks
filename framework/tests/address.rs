extern crate e2d2;
use e2d2::utils::*;
use std::net::Ipv4Addr;
use std::str::FromStr;

#[test]
fn address_inline() {
    let pfx = Ipv4Prefix::new(u32::from(Ipv4Addr::from_str("192.168.0.0").unwrap()), 16);
    assert!(pfx.in_range(u32::from(Ipv4Addr::from_str("192.168.0.1").unwrap()),));
    assert!(pfx.in_range(u32::from_be(16820416)));
    assert!(pfx.in_range(u32::from(Ipv4Addr::from_str("192.168.100.1").unwrap()),));
    assert!(pfx.in_range(u32::from(Ipv4Addr::from_str("192.168.2.1").unwrap()),));
    assert!(!pfx.in_range(u32::from(Ipv4Addr::from_str("192.163.0.1").unwrap()),));
    assert!(pfx.in_range(u32::from(Ipv4Addr::from_str("192.168.0.0").unwrap()),));
    assert!(pfx.in_range(u32::from(Ipv4Addr::from_str("192.168.255.255").unwrap()),));

    let pfx = Ipv4Prefix::new(u32::from(Ipv4Addr::from_str("192.168.1.2").unwrap()), 32);
    assert!(pfx.in_range(u32::from(Ipv4Addr::from_str("192.168.1.2").unwrap()),));
    assert!(!pfx.in_range(u32::from(Ipv4Addr::from_str("192.168.1.3").unwrap()),));

    let pfx = Ipv4Prefix::new(u32::from(Ipv4Addr::from_str("0.0.0.0").unwrap()), 0);
    assert!(pfx.in_range(u32::from(Ipv4Addr::from_str("192.168.1.2").unwrap()),));
    assert!(pfx.in_range(u32::from(Ipv4Addr::from_str("2.2.2.2").unwrap()),));

    let pfx = Ipv4Prefix::new(u32::from(Ipv4Addr::from_str("0.0.0.0").unwrap()), 0);
    assert!(pfx.in_range(u32::from(Ipv4Addr::from_str("192.168.1.2").unwrap()),));
    assert!(pfx.in_range(u32::from(Ipv4Addr::from_str("2.2.2.2").unwrap()),));
}
