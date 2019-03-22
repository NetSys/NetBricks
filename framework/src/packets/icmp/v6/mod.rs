use native::zcsi::MBuf;
use std::fmt;
use packets::{Packet, Header};
use packets::ip::v6::Ipv6Packet;

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

/// Type of ICMPv6 message
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
#[repr(C, packed)]
pub struct Icmpv6Type(pub u8);

impl Icmpv6Type {
    pub fn new(value: u8) -> Self {
        Icmpv6Type(value)
    }
}

/// Supported ICMPv6 message types
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub mod Icmpv6Types {
    use super::Icmpv6Type;

    pub const PacketTooBig: Icmpv6Type = Icmpv6Type(2);

    // NDP types
    pub const RouterSolicitation: Icmpv6Type = Icmpv6Type(133);
    pub const RouterAdvertisement: Icmpv6Type = Icmpv6Type(134);
    pub const NeighborSolicitation: Icmpv6Type = Icmpv6Type(135);
    pub const NeighborAdvertisement: Icmpv6Type = Icmpv6Type(136);
    pub const Redirect: Icmpv6Type = Icmpv6Type(137);
}

impl fmt::Display for Icmpv6Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                &Icmpv6Types::PacketTooBig => "Packet Too Big".to_string(),
                &Icmpv6Types::RouterSolicitation => "Router Solicitation".to_string(),
                &Icmpv6Types::RouterAdvertisement => "Router Advertisement".to_string(),
                &Icmpv6Types::NeighborSolicitation => "Neighbor Solicitation".to_string(),
                &Icmpv6Types::NeighborAdvertisement => "Neighbor Advertisement".to_string(),
                &Icmpv6Types::Redirect => "Redirect".to_string(),
                _ => format!("{}", self.0)
            }
        )
    }
}

/// ICMPv6 packet header
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

/// ICMPv6 packet payload
/// 
/// The ICMPv6 packet may contain a variable length payload. This 
/// is only the fixed portion. The variable length portion has to 
/// be parsed separately.
pub trait Icmpv6Payload {
    /// Returns the size of the fixed payload in bytes
    fn size() -> usize;
}

/// ICMPv6 unit payload `()`
impl Icmpv6Payload for () {
    fn size() -> usize {
        0
    }
}

/// Common behaviors shared by ICMPv6 packets
pub trait Icmpv6Packet<P: Icmpv6Payload>: Packet<Header=Icmpv6Header> {
    /// Returns the fixed payload
    fn payload(&self) -> &mut P;

    #[inline]
    fn msg_type(&self) -> Icmpv6Type {
        Icmpv6Type::new(self.header().msg_type)
    }

    #[inline]
    fn set_msg_type(&mut self, msg_type: Icmpv6Type) {
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

    #[inline]
    fn set_checksum(&mut self, checksum: u16) {
        // TODO: replace this with checksum calculation
        self.header().checksum = u16::to_be(checksum)
    }
}

/// ICMPv6 packet
pub struct Icmpv6<E: Ipv6Packet, P: Icmpv6Payload> {
    envelope: E,
    mbuf: *mut MBuf,
    offset: usize,
    header: *mut Icmpv6Header,
    payload: *mut P
}

/// ICMPv6 packet with unit payload
/// 
/// Use unit payload `()` when the payload type is not known yet.
/// 
/// # Example
/// 
/// ```
/// if ipv6.next_header() == NextHeaders::Icmpv6 {
///     let icmpv6 = ipv6.parse::<Icmpv6<()>>().unwrap();
/// }
/// ```
impl<E: Ipv6Packet> Icmpv6<E, ()> {
    /// Downcasts from unit payload to typed payload
    /// 
    /// # Example
    /// 
    /// ```
    /// if icmpv6.msg_type() == Icmpv6Types::RouterAdvertisement {
    ///     let advert = icmpv6.downcast::<RouterAdvertisement>();
    /// }
    /// ```
    pub fn downcast<P: Icmpv6Payload>(self) -> Icmpv6<E, P> {
        Icmpv6::<E, P>::from_packet(self.envelope, self.mbuf, self.offset, self.header)
    }
}

impl<E: Ipv6Packet> fmt::Display for Icmpv6<E, ()> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "type: {}, code: {}, checksum: 0x{:04x}",
            self.msg_type(),
            self.code(),
            self.checksum()
        )
    }
}

impl<E: Ipv6Packet> Icmpv6Packet<()> for Icmpv6<E, ()> {
    fn payload(&self) -> &mut () {
        unsafe { &mut (*self.payload) }
    }
}

impl<E: Ipv6Packet, P: Icmpv6Payload> Packet for Icmpv6<E, P> {
    type Header = Icmpv6Header;
    type Envelope = E;

    #[inline]
    fn from_packet(envelope: Self::Envelope,
                   mbuf: *mut MBuf,
                   offset: usize,
                   header: *mut Self::Header) -> Self {
        // TODO: should be a better way to do this
        let payload = envelope.get_mut_item::<P>(offset + Icmpv6Header::size());

        Icmpv6 {
            envelope,
            mbuf,
            offset,
            header,
            payload
        }
    }

    #[inline]
    fn envelope(&self) -> &Self::Envelope {
        &self.envelope
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

#[cfg(test)]
mod tests {
    use super::*;
    use packets::{RawPacket, Ethernet};
    use packets::ip::v6::Ipv6;
    use dpdk_test;

    #[rustfmt::skip]
    const ICMPV6_PACKET: [u8; 62] = [
        // ** ethernet header
        0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
        0x86, 0xDD,
        // ** IPv6 header
        0x60, 0x00, 0x00, 0x00,
        // payload length
        0x00, 0x08,
        0x3a,
        0xff,
        0xfe, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xd4, 0xf0, 0x45, 0xff, 0xfe, 0x0c, 0x66, 0x4b,
        0xff, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        // ** ICMPv6 header
        // type
        0x81,
        // code
        0x00,
        // checksum
        0xf5, 0x0c,
        // ** echo request
        0x00, 0x00, 0x00, 0x00
    ];

    #[test]
    fn parse_icmpv6_packet() {
        dpdk_test! {
            let packet = RawPacket::from_bytes(&ICMPV6_PACKET).unwrap();
            let ethernet = packet.parse::<Ethernet>().unwrap();
            let ipv6 = ethernet.parse::<Ipv6>().unwrap();
            let icmpv6 = ipv6.parse::<Icmpv6<Ipv6, ()>>().unwrap();

            assert_eq!(Icmpv6Type::new(0x81), icmpv6.msg_type());
            assert_eq!(0, icmpv6.code());
            assert_eq!(0xf50c, icmpv6.checksum());
        }
    }
}
