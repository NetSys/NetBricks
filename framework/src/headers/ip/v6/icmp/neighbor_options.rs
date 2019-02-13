use super::{EndOffset, Ipv6VarHeader};
use headers::mac::MacAddress;
use num::FromPrimitive;
use std::default::Default;
use std::fmt;
use std::marker::PhantomData;
use std::iter::Map;
use headers::ip::v6::icmp::router_advertisement::Icmpv6RouterAdvertisement;

#[derive(FromPrimitive, Debug, PartialEq)]
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

trait Icmpv6Option {
    fn option_type(&self) -> Icmpv6OptionType;
    fn option_length(&self) -> u8;
}

#[derive(Default)]
#[repr(C, packed)]
struct Icmpv6LinkLayerAddressOptionType {
    source_link_layer_address: MacAddress
}

impl Icmpv6LinkLayerAddressOption for Icmpv6LinkLayerAddressOptionType {
    fn source_link_layer_address(&self) -> MacAddress {
        self.source_link_layer_address
    }
}

impl Icmpv6LinkLayerAddressOptionType {

    #[inline]
    fn set_source_link_layer_address(&mut self, mac_addr: MacAddress)  {
        self.source_link_layer_address = mac_addr
    }
}

trait Icmpv6LinkLayerAddressOption: Icmpv6Option {
    fn option_type(&self) -> Icmpv6OptionType {
        Icmpv6OptionType::SourceLinkLayerAddress
    }

    fn option_length(&self) -> u8 {
        //length is always 8 bytes. we can move this to a map lookup if we need to
        8
    }
    fn source_link_layer_address(&self) -> MacAddress;
}

pub trait IPv6Optionable {
   // fn parse(&self) -> Map<Icmpv6OptionType, &Icmpv6Option>;
}
