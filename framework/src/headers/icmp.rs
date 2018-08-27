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

impl<T> IcmpV6Header<T>
    where T: IpHeader,
{
    #[inline]
    pub fn set_msg_type(&mut self, src: Ipv6Addr) {
        self.src_ip = src
    }

    #[inline]
    pub fn set_code(&mut self, code: u8) { self.code = code}

    #[inline]
    pub fn set_checksum(&mut self, checksum: u16) { self.checksum = checksum}
}