use native::zcsi::MBuf;
use std::fmt;
use packets::{Packet, Header};
use packets::ip::v6::Ipv6;

pub use self::ndp::*;
pub use self::ndp::options::*;
pub use self::ndp::router_advert::*;
pub use self::ndp::router_solicit::*;

pub mod ndp;

/*  From (https://tools.ietf.org/html/rfc4443)
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

/// type
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
#[repr(C, packed)]
pub struct Type(pub u8);

impl Type {
    pub fn new(value: u8) -> Self {
        Type(value)
    }
}

#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub mod Types {
    use super::Type;

    pub const PacketTooBig: Type = Type(2);

    pub const RouterSolicitation: Type = Type(133);
    pub const RouterAdvertisement: Type = Type(134);
    pub const NeighborSolicitation: Type = Type(135);
    pub const NeighborAdvertisement: Type = Type(136);
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                &Types::PacketTooBig => "Packet Too Big".to_string(),
                &Types::RouterSolicitation => "Router Solicitation".to_string(),
                &Types::RouterAdvertisement => "Router Advertisement".to_string(),
                &Types::NeighborSolicitation => "Neighbor Solicitation".to_string(),
                &Types::NeighborAdvertisement => "Neighbor Advertisement".to_string(),
                _ => format!("{}", self.0)
            }
        )
    }
}

/// icmpv6 header
#[derive(Default, Debug)]
#[repr(C, packed)]
pub struct Icmpv6Header {
    msg_type: u8,
    code: u8,
    checksum: u16
}

impl Header for Icmpv6Header {
    fn size() -> usize {
        4
    }
}

pub trait Icmpv6Payload {
    fn size() -> usize;
}

impl Icmpv6Payload for () {
    fn size() -> usize {
        0
    }
}

/// common fn all icmpv6 packets share
pub trait Icmpv6Packet<T: Icmpv6Payload>: Packet<Header=Icmpv6Header> {
    fn payload(&self) -> &mut T;

    #[inline]
    fn msg_type(&self) -> Type {
        Type::new(self.header().msg_type)
    }

    #[inline]
    fn set_msg_type(&mut self, msg_type: Type) {
        self.header().msg_type = msg_type.0
    }

    #[inline]
    fn code(&self) -> u8 {
        self.header().code
    }

    #[inline]
    fn set_code(&mut self, code: u8) {
        self.header().code = code
    }

    #[inline]
    fn checksum(&self) -> u16 {
        u16::from_be(self.header().checksum)
    }

    // TODO: replace this with checksum calculation
    #[inline]
    fn set_checksum(&mut self, checksum: u16) {
        self.header().checksum = u16::to_be(checksum)
    }
}

/// icmpv6 packet
pub struct Icmpv6<T: Icmpv6Payload> {
    mbuf: *mut MBuf,
    offset: usize,
    header: *mut Icmpv6Header,
    payload: *mut T,    // this is only the fixed part of the payload
    previous: Ipv6
}

impl Icmpv6<()> {
    pub fn downcast<T: Icmpv6Payload>(self) -> Icmpv6<T> {
        Icmpv6::<T>::from_packet(self.previous, self.mbuf, self.offset, self.header)
    }
}

impl fmt::Display for Icmpv6<()> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "type: {} code: {} checksum: 0x{:04x}",
            self.msg_type(),
            self.code(),
            self.checksum()
        )
    }
}

impl<T: Icmpv6Payload> Packet for Icmpv6<T> {
    type Header = Icmpv6Header;
    type PreviousPacket = Ipv6;

    #[inline]
    fn from_packet(previous: Self::PreviousPacket,
                   mbuf: *mut MBuf,
                   offset: usize,
                   header: *mut Self::Header) -> Self {
        // TODO: should be a better way to do this
        let payload = previous.get_mut_item::<T>(offset + Icmpv6Header::size());

        Icmpv6 {
            previous,
            mbuf,
            offset,
            header,
            payload
        }
    }

    #[inline]
    fn mbuf(&self) -> *mut MBuf {
        self.mbuf
    }

    #[inline]
    fn offset(&self) -> usize {
        self.offset
    }

    #[inline]
    fn header(&self) -> &mut Self::Header {
        unsafe { &mut (*self.header) }
    }

    #[inline]
    fn header_len(&self) -> usize {
        Self::Header::size()
    }
}

impl Icmpv6Packet<()> for Icmpv6<()> {
    fn payload(&self) -> &mut () {
        unsafe { &mut (*self.payload) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use packets::{RawPacket, Ethernet};
    use dpdk_test;
    use tests::ICMP_RTR_ADV_BYTES;

    #[test]
    fn str_from_icmpv6_packet() {
        dpdk_test! {
            let packet = RawPacket::from_bytes(&ICMP_RTR_ADV_BYTES).unwrap();
            let ethernet = packet.parse::<Ethernet>().unwrap();
            let ipv6 = ethernet.parse::<Ipv6>().unwrap();
            let icmpv6 = ipv6.parse::<Icmpv6<()>>().unwrap();

            assert_eq!("type: Router Advertisement code: 0 checksum: 0xf50c", icmpv6.to_string())
        }
    }
}
