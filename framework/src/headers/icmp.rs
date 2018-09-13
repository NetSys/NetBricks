use headers::ip::IpHeader;
use std::default::Default;
use std::fmt;
#[macro_use]
extern crate enum_primitive;
use enum_primitive::FromPrimitive;

/*
   ICMPv6 messages are contained in Ipv6 packets. The IPV6packet contains an IPv6 header followed by the
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

pub const ROUTER_ADVERTISEMENT: u8 = 134;
pub const NEIGHBOR_SOLICITATION: u8 = 135;
pub const NEIGHBOR_ADVERTISEMENT: u8 = 136;

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum IcmpMessageType {
    RouterAdvertisement = ROUTER_ADVERTISEMENT,
    NeighborSolicitation = NEIGHBOR_SOLICITATION,
    NeighborAdvertisement = NEIGHBOR_ADVERTISEMENT,
}
}

pub struct IcmpV6Header<T> {
    msg_type: IcmpMessageType,
    code: u8,
    checksum: u16,
}

impl Default<T> for IcmpV6Header<T>
where
    T: IpHeader,
{
    fn default() -> IcmpV6Header<T> {
        IcmpV6Header {
            msg_type: None,
            code: 0,
            checksum: 0,
        }
    }

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "msg_type: {} code: {} checksum: {}",
            self.msg_type(),
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
        54
    }

    #[inline]
    fn size() -> usize {
        32
    }

    #[inline]
    fn payload_size(&self, _: usize) -> usize {
        self.length() as usize - self.offset()
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
    pub fn set_msg_type(&mut self, msg_type: IcmpMessageType) {
        self.msg_type = msg_type
    }

    #[inline]
    pub fn msg_type(&self) -> IcmpMessageType { self.msg_type }

    #[inline]
    pub fn set_code(&mut self, code: u16) { self.code = u8::to_be(code) }

    #[inline]
    pub fn code(&self) { u8::from_be(self.code)}

    #[inline]
    pub fn checksum(&self) -> u16 {
        u16::from_be(self.csum)
    }

    #[inline]
    pub fn set_checksum(&mut self, csum: u16) {
        self.csum = u16::to_be(csum)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_icmp_message_types() {
        assert_eq!(IcmpMessageType::from_u8(134), Some(IcmpMessageType::RouterAdvertisement));
        assert_eq!(IcmpMessageType::from_u8(135), Some(IcmpMessageType::NeighborSolicitation));
        assert_eq!(IcmpMessageType::from_u8(136), Some(IcmpMessageType::NeighborAdvertisement));
        assert_eq!(IcmpMessageType::from_u8(4), None);
    }
}


