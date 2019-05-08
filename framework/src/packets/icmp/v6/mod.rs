use common::Result;
use native::zcsi::MBuf;
use packets::ip::v6::Ipv6Packet;
use packets::ip::ProtocolNumbers;
use packets::{buffer, checksum, Fixed, Header, Packet, ParseError};
use std::fmt;

pub use self::echo_reply::*;
pub use self::echo_request::*;
pub use self::ndp::neighbor_advert::*;
pub use self::ndp::neighbor_solicit::*;
pub use self::ndp::options::*;
pub use self::ndp::router_advert::*;
pub use self::ndp::router_solicit::*;
pub use self::ndp::*;
pub use self::too_big::*;

pub mod echo_reply;
pub mod echo_request;
pub mod ndp;
pub mod too_big;

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
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash)]
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
    pub const EchoRequest: Icmpv6Type = Icmpv6Type(128);
    pub const EchoReply: Icmpv6Type = Icmpv6Type(129);

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
                &Icmpv6Types::EchoRequest => "Echo Request".to_string(),
                &Icmpv6Types::EchoReply => "Echo Reply".to_string(),
                &Icmpv6Types::RouterSolicitation => "Router Solicitation".to_string(),
                &Icmpv6Types::RouterAdvertisement => "Router Advertisement".to_string(),
                &Icmpv6Types::NeighborSolicitation => "Neighbor Solicitation".to_string(),
                &Icmpv6Types::NeighborAdvertisement => "Neighbor Advertisement".to_string(),
                &Icmpv6Types::Redirect => "Redirect".to_string(),
                _ => format!("{}", self.0),
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
    checksum: u16,
}

impl Header for Icmpv6Header {}

/// ICMPv6 packet payload
///
/// The ICMPv6 packet may contain a variable length payload. This
/// is only the fixed portion. The variable length portion has to
/// be parsed separately.
pub trait Icmpv6Payload: Fixed + Default {
    /// Returns the ICMPv6 message type that corresponds to the payload
    fn msg_type() -> Icmpv6Type;
}

/// ICMPv6 unit payload `()`
impl Icmpv6Payload for () {
    fn msg_type() -> Icmpv6Type {
        // Unit payload does not have a type
        unreachable!();
    }
}

/// Common behaviors shared by ICMPv6 packets
pub trait Icmpv6Packet<E: Ipv6Packet, P: Icmpv6Payload>:
    Packet<Header = Icmpv6Header, Envelope = E>
{
    /// Returns a reference to the fixed payload
    fn payload(&self) -> &P;

    /// Returns a mutable reference to the fixed payload
    fn payload_mut(&mut self) -> &mut P;

    #[inline]
    fn msg_type(&self) -> Icmpv6Type {
        Icmpv6Type::new(self.header().msg_type)
    }

    #[inline]
    fn code(&self) -> u8 {
        self.header().code
    }

    #[inline]
    fn set_code(&mut self, code: u8) {
        self.header_mut().code = code
    }

    #[inline]
    fn checksum(&self) -> u16 {
        u16::from_be(self.header().checksum)
    }

    #[inline]
    fn compute_checksum(&mut self) {
        self.header_mut().checksum = 0;

        if let Ok(data) = buffer::read_slice(self.mbuf(), self.offset(), self.len()) {
            let data = unsafe { &(*data) };
            let pseudo_header_sum = self
                .envelope()
                .pseudo_header_sum(data.len() as u16, ProtocolNumbers::Icmpv6);
            let checksum = checksum::compute(pseudo_header_sum, data);
            self.header_mut().checksum = u16::to_be(checksum);
        } else {
            // we are reading till the end of buffer, should never run out
            unreachable!()
        }
    }
}

/// ICMPv6 packet
#[derive(Debug)]
pub struct Icmpv6<E: Ipv6Packet, P: Icmpv6Payload> {
    envelope: E,
    mbuf: *mut MBuf,
    offset: usize,
    header: *mut Icmpv6Header,
    payload: *mut P,
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
    ///     let advert = icmpv6.downcast::<RouterAdvertisement>().unwrap();
    /// }
    /// ```
    pub fn downcast<P: Icmpv6Payload>(self) -> Result<Icmpv6<E, P>> {
        Icmpv6::<E, P>::do_parse(self.envelope)
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

impl<E: Ipv6Packet, P: Icmpv6Payload> Icmpv6Packet<E, P> for Icmpv6<E, P> {
    fn payload(&self) -> &P {
        unsafe { &(*self.payload) }
    }

    fn payload_mut(&mut self) -> &mut P {
        unsafe { &mut (*self.payload) }
    }
}

impl<E: Ipv6Packet, P: Icmpv6Payload> Packet for Icmpv6<E, P> {
    type Header = Icmpv6Header;
    type Envelope = E;

    #[inline]
    fn envelope(&self) -> &Self::Envelope {
        &self.envelope
    }

    #[inline]
    fn envelope_mut(&mut self) -> &mut Self::Envelope {
        &mut self.envelope
    }

    #[doc(hidden)]
    #[inline]
    fn mbuf(&self) -> *mut MBuf {
        self.mbuf
    }

    #[inline]
    fn offset(&self) -> usize {
        self.offset
    }

    #[doc(hidden)]
    #[inline]
    fn header(&self) -> &Self::Header {
        unsafe { &(*self.header) }
    }

    #[doc(hidden)]
    #[inline]
    fn header_mut(&mut self) -> &mut Self::Header {
        unsafe { &mut (*self.header) }
    }

    #[inline]
    fn header_len(&self) -> usize {
        Self::Header::size()
    }

    #[doc(hidden)]
    #[inline]
    fn do_parse(envelope: Self::Envelope) -> Result<Self> {
        let mbuf = envelope.mbuf();
        let offset = envelope.payload_offset();
        let header = buffer::read_item::<Self::Header>(mbuf, offset)?;
        let payload = buffer::read_item::<P>(mbuf, offset + Self::Header::size())?;

        Ok(Icmpv6 {
            envelope,
            mbuf,
            offset,
            header,
            payload,
        })
    }

    #[doc(hidden)]
    #[inline]
    fn do_push(envelope: Self::Envelope) -> Result<Self> {
        let mbuf = envelope.mbuf();
        let offset = envelope.payload_offset();

        buffer::alloc(mbuf, offset, Self::Header::size() + P::size())?;
        let header = buffer::write_item::<Self::Header>(mbuf, offset, &Default::default())?;
        let payload =
            buffer::write_item::<P>(mbuf, offset + Self::Header::size(), &Default::default())?;

        unsafe {
            (*header).msg_type = P::msg_type().0;
        }

        Ok(Icmpv6 {
            envelope,
            mbuf,
            offset,
            header,
            payload,
        })
    }

    #[inline]
    fn remove(self) -> Result<Self::Envelope> {
        buffer::dealloc(self.mbuf, self.offset, self.header_len())?;
        Ok(self.envelope)
    }

    #[inline]
    default fn cascade(&mut self) {
        self.compute_checksum();
        self.envelope_mut().cascade();
    }

    #[inline]
    fn deparse(self) -> Self::Envelope {
        self.envelope
    }
}

/// An ICMPv6 message with parsed payload
pub enum Icmpv6Message<E: Ipv6Packet> {
    EchoRequest(Icmpv6<E, EchoRequest>),
    EchoReply(Icmpv6<E, EchoReply>),
    NeighborAdvertisement(Icmpv6<E, NeighborAdvertisement>),
    NeighborSolicitation(Icmpv6<E, NeighborSolicitation>),
    RouterAdvertisement(Icmpv6<E, RouterAdvertisement>),
    RouterSolicitation(Icmpv6<E, RouterSolicitation>),
    /// an ICMPv6 message with undefined payload
    Undefined(Icmpv6<E, ()>),
}

/// ICMPv6 helper functions for IPv6 packets
pub trait Icmpv6Parse {
    type Envelope: Ipv6Packet;

    /// Parses the payload as an ICMPv6 packet
    ///
    /// # Example
    ///
    /// ```
    /// match ipv6.parse_icmpv6()? {
    ///     Icmpv6Message::RouterAdvertisement(advert) => {
    ///         advert.set_router_lifetime(0);
    ///     },
    ///     Icmpv6Message::Undefined(icmpv6) => {
    ///         println!("undefined");
    ///     }
    /// }
    /// ```
    fn parse_icmpv6(self) -> Result<Icmpv6Message<Self::Envelope>>;
}

impl<T: Ipv6Packet> Icmpv6Parse for T {
    type Envelope = T;

    fn parse_icmpv6(self) -> Result<Icmpv6Message<Self::Envelope>> {
        if self.next_proto() == ProtocolNumbers::Icmpv6 {
            let icmpv6 = self.parse::<Icmpv6<Self::Envelope, ()>>()?;
            match icmpv6.msg_type() {
                Icmpv6Types::EchoRequest => {
                    let packet = icmpv6.downcast::<EchoRequest>()?;
                    Ok(Icmpv6Message::EchoRequest(packet))
                }
                Icmpv6Types::EchoReply => {
                    let packet = icmpv6.downcast::<EchoReply>()?;
                    Ok(Icmpv6Message::EchoReply(packet))
                }
                Icmpv6Types::NeighborAdvertisement => {
                    let packet = icmpv6.downcast::<NeighborAdvertisement>()?;
                    Ok(Icmpv6Message::NeighborAdvertisement(packet))
                }
                Icmpv6Types::NeighborSolicitation => {
                    let packet = icmpv6.downcast::<NeighborSolicitation>()?;
                    Ok(Icmpv6Message::NeighborSolicitation(packet))
                }
                Icmpv6Types::RouterAdvertisement => {
                    let packet = icmpv6.downcast::<RouterAdvertisement>()?;
                    Ok(Icmpv6Message::RouterAdvertisement(packet))
                }
                Icmpv6Types::RouterSolicitation => {
                    let packet = icmpv6.downcast::<RouterSolicitation>()?;
                    Ok(Icmpv6Message::RouterSolicitation(packet))
                }
                _ => Ok(Icmpv6Message::Undefined(icmpv6)),
            }
        } else {
            Err(ParseError::new("Packet is not ICMPv6").into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dpdk_test;
    use packets::ip::v6::Ipv6;
    use packets::{Ethernet, RawPacket};

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
        // unknown type
        0xFF,
        // code
        0x00,
        // checksum
        0x01, 0xf0,
        // data
        0x00, 0x00, 0x00, 0x00
    ];

    #[test]
    fn size_of_icmpv6_header() {
        assert_eq!(4, Icmpv6Header::size());
    }

    #[test]
    fn parse_icmpv6_packet() {
        dpdk_test! {
            let packet = RawPacket::from_bytes(&ICMPV6_PACKET).unwrap();
            let ethernet = packet.parse::<Ethernet>().unwrap();
            let ipv6 = ethernet.parse::<Ipv6>().unwrap();
            let icmpv6 = ipv6.parse::<Icmpv6<Ipv6, ()>>().unwrap();

            assert_eq!(Icmpv6Type::new(0xFF), icmpv6.msg_type());
            assert_eq!(0, icmpv6.code());
            assert_eq!(0x01f0, icmpv6.checksum());
        }
    }

    #[test]
    fn downcast_icmpv6() {
        use packets::icmp::v6::ndp::router_advert::tests::ROUTER_ADVERT_PACKET;

        dpdk_test! {
            let packet = RawPacket::from_bytes(&ROUTER_ADVERT_PACKET).unwrap();
            let ethernet = packet.parse::<Ethernet>().unwrap();
            let ipv6 = ethernet.parse::<Ipv6>().unwrap();
            let icmpv6 = ipv6.parse::<Icmpv6<Ipv6, ()>>().unwrap();
            let advert = icmpv6.downcast::<RouterAdvertisement>().unwrap();

            // check one accessor that belongs to `RouterAdvertisement`
            assert_eq!(64, advert.current_hop_limit());
        }
    }

    #[test]
    fn compute_checksum() {
        use packets::icmp::v6::ndp::router_advert::tests::ROUTER_ADVERT_PACKET;

        dpdk_test! {
            let packet = RawPacket::from_bytes(&ROUTER_ADVERT_PACKET).unwrap();
            let ethernet = packet.parse::<Ethernet>().unwrap();
            let ipv6 = ethernet.parse::<Ipv6>().unwrap();
            let mut icmpv6 = ipv6.parse::<Icmpv6<Ipv6, ()>>().unwrap();

            let expected = icmpv6.checksum();
            // no payload change but force a checksum recompute anyway
            icmpv6.cascade();
            assert_eq!(expected, icmpv6.checksum());
        }
    }

    #[test]
    fn matchable_icmpv6_packets() {
        dpdk_test! {
            let packet = RawPacket::from_bytes(&ICMPV6_PACKET).unwrap();
            let ethernet = packet.parse::<Ethernet>().unwrap();
            let ipv6 = ethernet.parse::<Ipv6>().unwrap();
            if let Ok(Icmpv6Message::Undefined(icmpv6)) = ipv6.parse_icmpv6() {
                assert_eq!(Icmpv6Type::new(0xFF), icmpv6.msg_type());
            } else {
                panic!("bad packet");
            }
        }
    }
}
