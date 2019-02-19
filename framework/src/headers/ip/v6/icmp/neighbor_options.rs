use headers::mac::MacAddress;
use std::collections::HashMap;
use std::fmt;
use std::net::Ipv6Addr;

#[derive(FromPrimitive, Debug, PartialEq, Hash, Eq, Clone, Copy)]
#[repr(u8)]
pub enum Icmpv6OptionType {
    SourceLinkLayerAddress = 1,
    TargetLinkLayerAddress = 2,
    PrefixInformation = 3,
    RedirectHeader = 4,
    MTU = 5,
}

impl fmt::Display for Icmpv6OptionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Icmpv6OptionType::SourceLinkLayerAddress => write!(f, "Source Link Layer Address"),
            Icmpv6OptionType::TargetLinkLayerAddress => write!(f, "Target Link Layer Address"),
            Icmpv6OptionType::PrefixInformation => write!(f, "Prefix Information"),
            Icmpv6OptionType::RedirectHeader => write!(f, "Redirect Header"),
            Icmpv6OptionType::MTU => write!(f, "MTU"),
        }
    }
}
#[derive(Debug)]
#[repr(C)]
pub struct Icmpv6MtuOption {
    prefix_length: u8,
    reserved1: u8,
    valid_lifetime: u32,
    preferred_lifetime: u32,
    reserved2: u32,
    prefix_information: Ipv6Addr,
}

#[derive(Debug)]
#[repr(C)]
pub struct Icmpv6RedirectHeaderOption {
    reserved1: u16,
    reserved2: u32,
    ipheader_data: u32,
}

#[derive(Debug)]
#[repr(C)]
pub struct Icmpv6PrefixInformationOption {
    prefix_length: u8,
    reserved1: u8,
    valid_lifetime: u32,
    preferred_lifetime: u32,
    reserved2: u32,
    prefix_information: Ipv6Addr,
}

#[derive(Debug)]
#[repr(C)]
pub enum Icmpv6Option {
    SourceLinkLayerAddress(MacAddress),
    TargetLinkLayerAddress(MacAddress),
    Mtu(Icmpv6MtuOption),
    RedirectHeader(Icmpv6RedirectHeaderOption),
    PrefixInformation(Icmpv6PrefixInformationOption),
}

pub trait IPv6Optionable {
    fn parse_options(&self, payload_len: u16) -> HashMap<Icmpv6OptionType, Icmpv6Option>;
}
