use super::{IcmpMessageType, Icmpv6Header, Icmpv6Neighbor};
use headers::{EndOffset, Ipv6VarHeader};
use std::default::Default;
use std::fmt;
use std::marker::PhantomData;
use std::net::Ipv6Addr;

/*
  ICMPv6 messages are contained in IPv6 packets. The IPv6 packet contains an IPv6 header followed by the
  payload which contains the ICMPv6 message.

  From (https://tools.ietf.org/html/rfc4861)
  The ICMPv6 Neighbor Solicitation Messages have the following general format:

      0                   1                   2                   3
      0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
     +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
     |     Type      |     Code      |          Checksum             |
     +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
     |                           Reserved                            |
     +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
     |                                                               |
     +                                                               +
     |                                                               |
     +                       Target Address                          +
     |                                                               |
     +                                                               +
     |                                                               |
     +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
     |   Options ...
     +-+-+-+-+-+-+-+-+-+-+-+-

    IP Fields:

      Source Address
                     Either an address assigned to the interface from
                     which this message is sent or (if Duplicate Address
                     Detection is in progress [ADDRCONF]) the
                     unspecified address.
      Destination Address
                     Either the solicited-node multicast address
                     corresponding to the target address, or the target
                     address.
      Hop Limit      255

   ICMP Fields:

      Type           135

      Code           0

      Checksum       The ICMP checksum.  See [ICMPv6].

      Reserved       This field is unused.  It MUST be initialized to
                     zero by the sender and MUST be ignored by the
                     receiver.

      Target Address The IP address of the target of the solicitation.
                     It MUST NOT be a multicast address.

   Possible options:

      Source link-layer address
                     The link-layer address for the sender.  MUST NOT be
                     included when the source IP address is the
                     unspecified address.  Otherwise, on link layers
                     that have addresses this option MUST be included in
                     multicast solicitations and SHOULD be included in
                     unicast solicitations.

      Future versions of this protocol may define new option types.
      Receivers MUST silently ignore any options they do not recognize
      and continue processing the message.
*/

#[derive(Debug)]
#[repr(C)]
pub struct Icmpv6NeighborSolicitation<T>
where
    T: Ipv6VarHeader,
{
    icmp_neighbor: Icmpv6Neighbor<T>,
    _parent: PhantomData<T>,
}

impl<T> Default for Icmpv6NeighborSolicitation<T>
where
    T: Ipv6VarHeader,
{
    fn default() -> Icmpv6NeighborSolicitation<T> {
        Icmpv6NeighborSolicitation {
            icmp_neighbor: Icmpv6Neighbor {
                icmp: Icmpv6Header {
                    msg_type: IcmpMessageType::NeighborSolicitation as u8,
                    ..Default::default()
                },
                ..Default::default()
            },
            _parent: PhantomData,
        }
    }
}

impl<T> fmt::Display for Icmpv6NeighborSolicitation<T>
where
    T: Ipv6VarHeader,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "msg_type: {} code: {} checksum: {}, reserved_flags {}, target_addr {}, options {}",
            self.icmp_neighbor.msg_type().unwrap(),
            self.icmp_neighbor.code(),
            self.icmp_neighbor.checksum(),
            self.icmp_neighbor.reserved_flags(),
            self.icmp_neighbor.target_addr(),
            self.icmp_neighbor.options()
        )
    }
}

impl<T> EndOffset for Icmpv6NeighborSolicitation<T>
where
    T: Ipv6VarHeader,
{
    type PreviousHeader = T;

    #[inline]
    fn offset(&self) -> usize {
        // ICMPv6 Header for Neighbor Solicitation Msg (Type + Code + Checksum + Options)
        // is always 8 bytes: (8 + 8 + 16 + 32) / 8 = 8
        8
    }

    #[inline]
    fn size() -> usize {
        // ICMPv6 Header is always 8 bytes so size = offset
        8
    }

    #[inline]
    fn payload_size(&self, hint: usize) -> usize {
        // There is no payload size in the ICMPv6 header
        hint - self.offset()
    }

    #[inline]
    fn check_correct(&self, _prev: &T) -> bool {
        true
    }
}

impl<T> Icmpv6NeighborSolicitation<T>
where
    T: Ipv6VarHeader,
{
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn msg_type(&self) -> Option<IcmpMessageType> {
        self.icmp_neighbor.msg_type()
    }

    #[inline]
    pub fn code(&self) -> u8 {
        self.icmp_neighbor.code()
    }

    #[inline]
    pub fn checksum(&self) -> u16 {
        self.icmp_neighbor.checksum()
    }

    #[inline]
    pub fn reserved_flags(&self) -> u32 {
        self.icmp_neighbor.reserved_flags()
    }

    #[inline]
    pub fn options(&self) -> u32 {
        u32::from_be(self.icmp_neighbor.options())
    }

    #[inline]
    pub fn target_addr(&self) -> Ipv6Addr {
        self.icmp_neighbor.target_addr()
    }
}
