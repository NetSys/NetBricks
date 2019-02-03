use super::{EndOffset, Ipv6VarHeader};
use headers::mac::MacAddress;
use num::FromPrimitive;
use std::default::Default;
use std::fmt;
use std::marker::PhantomData;

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

#[derive(Debug, Clone)]
#[repr(C, packed)]
pub struct Icmpv6Option<T>
where
    T: Ipv6VarHeader,
{
    option_type: u8,
    option_length: u8,
    _parent: PhantomData<T>,
}

impl<T> Default for Icmpv6Option<T>
where
    T: Ipv6VarHeader,
{
    fn default() -> Icmpv6Option<T> {
        Icmpv6Option {
            option_type: 0,
            option_length: 0,
            _parent: PhantomData,
        }
    }
}

impl<T> fmt::Display for Icmpv6Option<T>
where
    T: Ipv6VarHeader,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "option_type: {} option_length: {}",
            self.option_type().unwrap(),
            self.option_length()
        )
    }
}

impl<T> EndOffset for Icmpv6Option<T>
where
    T: Ipv6VarHeader,
{
    type PreviousHeader = T;

    #[inline]
    fn offset(&self) -> usize {
        // ICMPv6 Option(Type + Length) is always 2 bytes: (8 + 8) / 8 = 2
        2
    }

    #[inline]
    fn size() -> usize {
        // Icmpv6Option Header is always 4 bytes so size = offset
        2
    }

    #[inline]
    fn payload_size(&self, hint: usize) -> usize {
        // There is no payload size in the ICMPv6 header
        hint - self.offset()
    }

    #[inline]
    fn check_correct(&self, _prev: &T) -> bool {
        true
    }
}

impl<T> Icmpv6Option<T>
where
    T: Ipv6VarHeader,
{
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn option_type(&self) -> Option<Icmpv6OptionType> {
        FromPrimitive::from_u8(self.option_type)
    }

    #[inline]
    pub fn option_length(&self) -> u8 {
        self.option_length
    }
}

#[derive(Debug, Clone)]
#[repr(C, packed)]
pub struct Icmpv6RouterAdvertisementOption<T>
where
    T: Ipv6VarHeader,
{
    pub icmp_option: Icmpv6Option<T>,
    pub source_link_layer_address: MacAddress,
    pub _parent: PhantomData<T>,
}

impl<T> Default for Icmpv6RouterAdvertisementOption<T>
where
    T: Ipv6VarHeader,
{
    fn default() -> Icmpv6RouterAdvertisementOption<T> {
        Icmpv6RouterAdvertisementOption {
            icmp_option: Icmpv6Option {
                ..Default::default()
            },
            source_link_layer_address: MacAddress {
                addr: [0, 0, 0, 0, 0, 0],
            },
            _parent: PhantomData,
        }
    }
}

impl<T> fmt::Display for Icmpv6RouterAdvertisementOption<T>
where
    T: Ipv6VarHeader,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "icmp_option: {} source_link_layer_address: {}",
            self.icmp_option, self.source_link_layer_address
        )
    }
}

impl<T> EndOffset for Icmpv6RouterAdvertisementOption<T>
where
    T: Ipv6VarHeader,
{
    type PreviousHeader = T;

    #[inline]
    fn offset(&self) -> usize {
        // ICMPv6 Neighbor Option(Source Link Layer Address) is always 1 byte: 8 / 8 = 1
        1
    }

    #[inline]
    fn size() -> usize {
        // ICMPv6 Neighbor Option(Source Link Layer Address) is always 1 byte so size = offset
        1
    }

    #[inline]
    fn payload_size(&self, hint: usize) -> usize {
        // There is no payload size in the Neighbor Option
        hint - self.offset()
    }

    #[inline]
    fn check_correct(&self, _prev: &T) -> bool {
        true
    }
}

impl<T> Icmpv6RouterAdvertisementOption<T>
where
    T: Ipv6VarHeader,
{
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn source_link_layer_address(&self) -> Option<MacAddress> {
        if self.icmp_option.option_type().unwrap() == Icmpv6OptionType::SourceLinkLayerAddress {
            Some(self.source_link_layer_address) }
        else {
            None
        }
    }
}
