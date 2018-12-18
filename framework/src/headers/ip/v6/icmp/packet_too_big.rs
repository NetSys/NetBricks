use super::{IcmpMessageType, Icmpv6Header};
use headers::{CalcChecksums, EndOffset, Ipv6VarHeader, IPV6_MIN_MTU};
use std::default::Default;
use std::fmt;
use std::marker::PhantomData;

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
  |                             MTU                               |
  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
  |                    As much of invoking packet                 |
  +               as possible without the ICMPv6 packet           +
  |               exceeding the minimum IPv6 MTU [IPv6]           |

  Type           2

  Code           Set to 0 (zero) by the originator and ignored by the
  receiver.

  MTU            The Maximum Transmission Unit of the next-hop link.

  A Packet Too Big MUST be sent by a router in response to a packet
  that it cannot forward because the packet is larger than the MTU of
  the outgoing link.  The information in this message is used as part
  of the Path MTU Discovery process [PMTU].

  Originating a Packet Too Big Message makes an exception to one of the
  rules as to when to originate an ICMPv6 error message.  Unlike other
  messages, it is sent in response to a packet received with an IPv6
  multicast destination address, or with a link-layer multicast or
  link-layer broadcast address.
*/

// Constants
pub const IPV6_TOO_BIG_PAYLOAD_LEN: u16 = 1240; // MTU MIN - v6 size of 40;

/// Payload Size after ICMP TOO_BIG Header
/// 1294 - MacHeader::size() - Ipv6Header::size() - Self::size()
pub const ICMPV6_TOO_BIG_PAYLOAD_SIZE: usize = 1232;

#[derive(Debug)]
#[repr(C, packed)]
pub struct Icmpv6PktTooBig<T>
where
    T: Ipv6VarHeader,
{
    pub icmp: Icmpv6Header<T>,
    mtu: u32,
    _parent: PhantomData<T>,
}

impl<T> Default for Icmpv6PktTooBig<T>
where
    T: Ipv6VarHeader,
{
    fn default() -> Icmpv6PktTooBig<T> {
        Icmpv6PktTooBig {
            icmp: Icmpv6Header {
                msg_type: IcmpMessageType::PacketTooBig as u8,
                ..Default::default()
            },
            mtu: u32::to_be(IPV6_MIN_MTU),
            _parent: PhantomData,
        }
    }
}

impl<T> fmt::Display for Icmpv6PktTooBig<T>
where
    T: Ipv6VarHeader,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "msg_type: {} code: {} checksum: {}, mtu: {}",
            self.msg_type().unwrap(),
            self.code(),
            self.checksum(),
            self.mtu()
        )
    }
}

impl<T> EndOffset for Icmpv6PktTooBig<T>
where
    T: Ipv6VarHeader,
{
    type PreviousHeader = T;

    #[inline]
    fn offset(&self) -> usize {
        // ICMPv6 Header for Packet Too Big Msg (Type + Code + Checksum + MTU)
        // is always 8 bytes: (8 + 8 + 16 + 32) / 8 = 8
        8
    }

    #[inline]
    fn size() -> usize {
        // ICMPv6 Header is always 8 bytes so size = offset
        8
    }

    #[inline]
    fn payload_size(&self, _hint: usize) -> usize {
        // We have to include as much of the offending original
        // packet as possible, hence we will always max to MTU, not
        // including the current header
        ICMPV6_TOO_BIG_PAYLOAD_SIZE
    }

    #[inline]
    fn check_correct(&self, _prev: &T) -> bool {
        true
    }
}

impl<T> Icmpv6PktTooBig<T>
where
    T: Ipv6VarHeader,
{
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn msg_type(&self) -> Option<IcmpMessageType> {
        self.icmp.msg_type()
    }

    #[inline]
    pub fn code(&self) -> u8 {
        self.icmp.code
    }

    #[inline]
    pub fn checksum(&self) -> u16 {
        self.icmp.checksum()
    }

    #[inline]
    pub fn mtu(&self) -> u32 {
        u32::from_be(self.mtu)
    }

    #[inline]
    pub fn set_mtu(&mut self, mtu: u32) {
        self.mtu = u32::to_be(mtu)
    }
}
