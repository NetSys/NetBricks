pub use self::v4::Ipv4Cidr;
pub use self::v6::Ipv6Cidr;
use failure::Fail;

pub mod v4;
pub mod v6;

#[derive(Debug, Fail)]
#[fail(display = "Failed to parse CIDR: {}", _0)]
pub struct CidrParseError((String));

pub trait Cidr: Sized {
    type Addr;

    fn address(&self) -> Self::Addr;
    fn length(&self) -> usize;
    fn new(address: Self::Addr, length: usize) -> Result<Self, CidrParseError>;
    fn contains(&self, address: Self::Addr) -> bool;
}
