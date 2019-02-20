use headers::mac::MacAddress;
use std::collections::HashMap;
use std::fmt;
use std::net::Ipv6Addr;

#[derive(FromPrimitive, Debug, PartialEq, Hash, Eq, Clone, Copy)]
#[repr(u8)]
pub enum NDPOptionType {
    SourceLinkLayerAddress = 1,
    TargetLinkLayerAddress = 2,
    PrefixInformation = 3,
    RedirectHeader = 4,
    MTU = 5,
}

impl fmt::Display for NDPOptionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            NDPOptionType::SourceLinkLayerAddress => write!(f, "Source Link Layer Address"),
            NDPOptionType::TargetLinkLayerAddress => write!(f, "Target Link Layer Address"),
            NDPOptionType::PrefixInformation => write!(f, "Prefix Information"),
            NDPOptionType::RedirectHeader => write!(f, "Redirect Header"),
            NDPOptionType::MTU => write!(f, "MTU"),
        }
    }
}
#[derive(Debug)]
#[repr(C)]
pub struct NDPMtuOption {
    prefix_length: u8,
    reserved1: u8,
    valid_lifetime: u32,
    preferred_lifetime: u32,
    reserved2: u32,
    prefix_information: Ipv6Addr,
}

#[derive(Debug)]
#[repr(C)]
pub struct NDPRedirectHeaderOption {
    reserved1: u16,
    reserved2: u32,
    ipheader_data: u32,
}

#[derive(Debug)]
#[repr(C)]
pub struct NDPPrefixInformationOption {
    prefix_length: u8,
    reserved1: u8,
    valid_lifetime: u32,
    preferred_lifetime: u32,
    reserved2: u32,
    prefix_information: Ipv6Addr,
}

#[derive(Debug)]
#[repr(C)]
pub enum NDPOption {
    SourceLinkLayerAddress(MacAddress),
    TargetLinkLayerAddress(MacAddress),
    Mtu(NDPMtuOption),
    RedirectHeader(NDPRedirectHeaderOption),
    PrefixInformation(NDPPrefixInformationOption),
}

pub trait NDPOptionParser {
    fn parse_options(&self, payload_len: u16) -> HashMap<NDPOptionType, NDPOption>;
}
