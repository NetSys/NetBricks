use headers::ip::IpHeader;
use std::default::Default;
use super::EndOffset;
use std::marker::PhantomData;
use num::FromPrimitive;
use std::fmt;

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
#[repr(u16)]
pub enum IcmpMessageType {
    RouterAdvertisement = 134,
    NeighborSolicitation = 135,
    NeighborAdvertisement = 136,
}

impl fmt::Display for IcmpMessageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IcmpMessageType::RouterAdvertisement => write!(f, "Router Advertisement"),
            IcmpMessageType::NeighborSolicitation => write!(f, "Neighbor Solicitation"),
            IcmpMessageType::NeighborAdvertisement => write!(f, "Neighbor Advertisement"),
        }
    }
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct IcmpV6Header<T> {
    msg_type: u8,
    code: u8,
    checksum: u16,
    _parent: PhantomData<T>,
}

impl<T> Default for IcmpV6Header<T>
    where
        T: IpHeader,
{
    fn default() -> IcmpV6Header<T> {
        IcmpV6Header {
            msg_type: 0,
            code: 0,
            checksum: 0,
            _parent: PhantomData
        }
    }
}

impl<T> fmt::Display for IcmpV6Header<T>
    where
        T: IpHeader,
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

impl<T> EndOffset for IcmpV6Header<T>
    where
        T: IpHeader,
{
    type PreviousHeader = T;

    #[inline]
    fn offset(&self) -> usize {
        // ICMPv6 Header(Type + Code + Checksum) is always 4 bytes: (8 + 8 + 16) / 8 = 4
        4
    }

    #[inline]
    fn size() -> usize {
        // ICMPv6 Header is always 4 bytes so size = offset
        4
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

impl<T> IcmpV6Header<T>
    where
        T: IpHeader,
{

    #[inline]
    pub fn msg_type(&self) -> Option<IcmpMessageType> {
        FromPrimitive::from_u8(u8::from_be(self.msg_type))
    }

    #[inline]
    pub fn set_msg_type(&mut self, msg_type: IcmpMessageType) {
        self.msg_type = u8::to_be(msg_type as u8)
    }

    #[inline]
    pub fn set_code(&mut self, code: u8) { self.code = u8::to_be(code) }

    #[inline]
    pub fn code(&self) -> u8{ u8::from_be(self.code)}

    #[inline]
    pub fn checksum(&self) -> u16 {
        u16::from_be(self.checksum)
    }

    #[inline]
    pub fn set_checksum(&mut self, csum: u16) {
        self.checksum = u16::to_be(csum)
    }
}
