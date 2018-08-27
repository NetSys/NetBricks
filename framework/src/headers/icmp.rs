use headers::ip::IpHeader;
use std::default::Default;
use std::fmt;

pub struct IcmpV6Header<T> {
    msg_type: u8,
    code: u8,
    checksum: u16
}

pub const ROUTER_ADVERTISEMENT: int = 134;
pub const NEIGHBOR_SOLICITATION: int = 135;
pub const NEIGHBOR_ADVERTISEMENT: int = 136;

impl Default<T> for IcmpV6Header<T>
    where T: IpHeader
{
    fn default() -> IcmpV6Header<T> {
        IcmpHeader {
            msg_type: 0,
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
