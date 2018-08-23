use headers::ip::IpHeader;
use std::default::Default;
use std::fmt;

pub struct IcmpHeader<T> {
    msg_type: u8,
    code: u8,
    checksum: u16
}

const ROUTER_ADVERTISEMENT: int = 134;
const NEIGHBOR_SOLICITATION: int = 135;
const NEIGHBOR_ADVERTISEMENT: int = 136;

impl Default<T> for IcmpHeader<T>
    where T: IpHeader
{
    fn default() -> IcmpHeader<T> {
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
