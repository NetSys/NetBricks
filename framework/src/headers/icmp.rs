use headers::ip::IpHeader;
use std::default::Default;
use std::fmt;
#[macro_use]
extern crate enum_primitive;
use enum_primitive::FromPrimitive;

pub const ROUTER_ADVERTISEMENT: int = 134;
pub const NEIGHBOR_SOLICITATION: int = 135;
pub const NEIGHBOR_ADVERTISEMENT: int = 136;

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
    checksum: u16
}

impl Default<T> for IcmpV6Header<T>
    where T: IpHeader
{
    fn default() -> IcmpV6Header<T> {
        IcmpHeader {
            code: 0,
            checksum: 0
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

impl<T> IcmpV6Header<T>
    where T: IpHeader,
{
    #[inline]
    pub fn set_msg_type(&mut self, msg_type: IcmpMessageType) {
        self.msg_type = msg_type;
    }

    #[inline]
    pub fn set_checksum(&mut self, checksum: u16) { self.checksum = checksum}
}

#[cfg(test)]
mod tests  {

    #[test]
    fn test_icmp_message_types() {
        assert_eq!(IcmpMessageType::from_i32(134), Some(IcmpMessageType::RouterAdvertisement));
        assert_eq!(IcmpMessageType::from_i32(135), Some(IcmpMessageType::NeighborSolicitation));
        assert_eq!(IcmpMessageType::from_i32(136), Some(IcmpMessageType::NeighborAdvertisement));
        assert_eq!(IcmpMessageType::from_i32(4), None);
    }
}