use super::{IcmpMessageType, Icmpv6Header};
use headers::{CalcChecksums, EndOffset, Ipv6VarHeader};
use std::default::Default;
use std::fmt;
use std::marker::PhantomData;
use std::net::Ipv6Addr;

#[derive(Debug)]
#[repr(C)]
pub struct NDPMessage<T>
where
    T: Ipv6VarHeader,
{
    pub icmp: Icmpv6Header<T>,
    pub reserved_flags: u32,
    pub target_addr: Ipv6Addr,
    pub _parent: PhantomData<T>,
}

impl<T> Default for NDPMessage<T>
where
    T: Ipv6VarHeader,
{
    fn default() -> NDPMessage<T> {
        NDPMessage {
            icmp: Icmpv6Header {
                ..Default::default()
            },

            reserved_flags: 0,
            target_addr: Ipv6Addr::UNSPECIFIED,
            _parent: PhantomData,
        }
    }
}

impl<T> fmt::Display for NDPMessage<T>
where
    T: Ipv6VarHeader,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "msg_type: {} code: {} checksum: {}, reserved_flags {}, target_addr {}",
            self.msg_type().unwrap(),
            self.code(),
            self.checksum(),
            self.reserved_flags(),
            self.target_addr(),
        )
    }
}

impl<T> EndOffset for NDPMessage<T>
where
    T: Ipv6VarHeader,
{
    type PreviousHeader = T;

    #[inline]
    fn offset(&self) -> usize {
        // ICMPv6 Header for Packet Too Big Msg (Type + Code + Checksum + MTU)
        // is always 8 bytes: (8 + 8 + 16 + 32) / 8 = 8
        8
    }

    #[inline]
    fn size() -> usize {
        // ICMPv6 Header is always 8 bytes so size = offset
        8
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

impl<T> NDPMessage<T>
where
    T: Ipv6VarHeader,
{
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn msg_type(&self) -> Option<IcmpMessageType> {
        self.icmp.msg_type()
    }

    #[inline]
    pub fn code(&self) -> u8 {
        self.icmp.code()
    }

    #[inline]
    pub fn checksum(&self) -> u16 {
        self.icmp.checksum()
    }

    #[inline]
    pub fn reserved_flags(&self) -> u32 {
        self.reserved_flags
    }

    #[inline]
    pub fn target_addr(&self) -> Ipv6Addr {
        self.target_addr
    }
}
