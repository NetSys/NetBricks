pub use self::ndp::*;
pub use self::neighbor_advertisement::*;
pub use self::neighbor_options::*;
pub use self::neighbor_solicitation::*;
pub use self::packet_too_big::*;
pub use self::router_advertisement::*;
use super::{EndOffset, Ipv6VarHeader};
use headers::CalcChecksums;
use num::FromPrimitive;
use std::default::Default;
use std::fmt;
use std::marker::PhantomData;

mod ndp;
mod neighbor_advertisement;
mod neighbor_options;
mod neighbor_solicitation;
mod packet_too_big;
mod router_advertisement;

/*
  ICMPv6 messages are contained in IPv6 packets. The IPv6 packet contains an IPv6 header followed by the
  payload which contains the ICMPv6 message.

  From (https://tools.ietf.org/html/rfc4443)
  The ICMPv6 messages have the following general format:

  0                   1                   2                   3
  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
  |     Type      |     Code      |          Checksum             |
  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
  |                                                               |
  +                         Message Body                          +
  |                                                               |

  The type field indicates the type of the message.  Its value
  determines the format of the remaining data.

  The code field depends on the message type.  It is used to create an
  additional level of message granularity.

  The checksum field is used to detect data corruption in the ICMPv6
  message and parts of the IPv6 header.
*/

#[derive(FromPrimitive, Debug, PartialEq)]
#[repr(u8)]
pub enum IcmpMessageType {
    PacketTooBig = 2,
    RouterAdvertisement = 134,
    NeighborSolicitation = 135,
    NeighborAdvertisement = 136,
}

impl fmt::Display for IcmpMessageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IcmpMessageType::PacketTooBig => write!(f, "Packet Too Big"),
            IcmpMessageType::RouterAdvertisement => write!(f, "Router Advertisement"),
            IcmpMessageType::NeighborSolicitation => write!(f, "Neighbor Solicitation"),
            IcmpMessageType::NeighborAdvertisement => write!(f, "Neighbor Advertisement"),
        }
    }
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct Icmpv6Header<T>
where
    T: Ipv6VarHeader,
{
    msg_type: u8,
    code: u8,
    checksum: u16,
    _parent: PhantomData<T>,
}

impl<T> Default for Icmpv6Header<T>
where
    T: Ipv6VarHeader,
{
    fn default() -> Icmpv6Header<T> {
        Icmpv6Header {
            msg_type: 0,
            code: 0,
            checksum: 0,
            _parent: PhantomData,
        }
    }
}

impl<T> fmt::Display for Icmpv6Header<T>
where
    T: Ipv6VarHeader,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "msg_type: {} code: {} checksum: {}",
            self.msg_type().unwrap(),
            self.code(),
            self.checksum()
        )
    }
}

impl<T> EndOffset for Icmpv6Header<T>
where
    T: Ipv6VarHeader,
{
    type PreviousHeader = T;

    #[inline]
    fn offset(&self) -> usize {
        // ICMPv6 Header(Type + Code + Checksum + Curr Hop Limit + Flags) is always 6 bytes: (8 + 8 + 16 + 8 + 8) / 8 = 6
        6
    }

    #[inline]
    fn size() -> usize {
        // ICMPv6 Header is always 4 bytes so size = offset
        6
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

impl<T> Icmpv6Header<T>
where
    T: Ipv6VarHeader,
{
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn msg_type(&self) -> Option<IcmpMessageType> {
        FromPrimitive::from_u8(self.msg_type)
    }

    #[inline]
    pub fn set_msg_type(&mut self, msg_type: IcmpMessageType) {
        self.msg_type = msg_type as u8
    }

    #[inline]
    pub fn set_code(&mut self, code: u8) {
        self.code = code
    }

    #[inline]
    pub fn code(&self) -> u8 {
        self.code
    }
}

impl<T> CalcChecksums for Icmpv6Header<T>
where
    T: Ipv6VarHeader,
{
    #[inline]
    fn checksum(&self) -> u16 {
        u16::from_be(self.checksum)
    }

    #[inline]
    fn set_checksum(&mut self, checksum: u16) {
        self.checksum = u16::to_be(checksum)
    }
}
