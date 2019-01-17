use super::{Icmpv6Neighbor, Icmpv6Header, IcmpMessageType};
use headers::{EndOffset, Ipv6VarHeader};
use std::default::Default;
use std::fmt;
use std::marker::PhantomData;
use std::net::Ipv6Addr;
use utils::*;

/*
  ICMPv6 messages are contained in IPv6 packets. The IPv6 packet contains an IPv6 header followed by the
  payload which contains the ICMPv6 message.

  From (https://tools.ietf.org/html/rfc4861)
  The ICMPv6 Neighbor Advertisement Messages have the following general format:

       0                   1                   2                   3
       0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
      +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
      |     Type      |     Code      |          Checksum             |
      +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
      |R|S|O|                     Reserved                            |
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
                     An address assigned to the interface from which the
                     advertisement is sent.
      Destination Address
                     For solicited advertisements, the Source Address of
                     an invoking Neighbor Solicitation or, if the
                     solicitation's Source Address is the unspecified
                     address, the all-nodes multicast address.

                     For unsolicited advertisements typically the all-
                     nodes multicast address.

      Hop Limit      255

   ICMP Fields:

      Type           136

      Code           0

      Checksum       The ICMP checksum.  See [ICMPv6].

      R              Router flag.  When set, the R-bit indicates that
                     the sender is a router.  The R-bit is used by
                     Neighbor Unreachability Detection to detect a
                     router that changes to a host.

      S              Solicited flag.  When set, the S-bit indicates that
                     the advertisement was sent in response to a
                     Neighbor Solicitation from the Destination address.
                     The S-bit is used as a reachability confirmation
                     for Neighbor Unreachability Detection.  It MUST NOT
                     be set in multicast advertisements or in
                     unsolicited unicast advertisements.

      O              Override flag.  When set, the O-bit indicates that
                     the advertisement should override an existing cache
                     entry and update the cached link-layer address.
                     When it is not set the advertisement will not
                     update a cached link-layer address though it will
                     update an existing Neighbor Cache entry for which
                     no link-layer address is known.  It SHOULD NOT be
                     set in solicited advertisements for anycast
                     addresses and in solicited proxy advertisements.
                     It SHOULD be set in other solicited advertisements
                     and in unsolicited advertisements.

      Reserved       29-bit unused field.  It MUST be initialized to
                     zero by the sender and MUST be ignored by the
                     receiver.

      Target Address
                     For solicited advertisements, the Target Address
                     field in the Neighbor Solicitation message that
                     prompted this advertisement.  For an unsolicited
                     advertisement, the address whose link-layer address
                     has changed.  The Target Address MUST NOT be a
                     multicast address.

   Possible options:

      Target link-layer address
                     The link-layer address for the target, i.e., the
                     sender of the advertisement.  This option MUST be
                     included on link layers that have addresses when
                     responding to multicast solicitations.  When
                     responding to a unicast Neighbor Solicitation this
                     option SHOULD be included.

                     The option MUST be included for multicast
                     solicitations in order to avoid infinite Neighbor
                     Solicitation "recursion" when the peer node does
                     not have a cache entry to return a Neighbor
                     Advertisements message.  When responding to unicast
                     solicitations, the option can be omitted since the
                     sender of the solicitation has the correct link-
                     layer address; otherwise, it would not be able to
                     send the unicast solicitation in the first place.
                     However, including the link-layer address in this
                     case adds little overhead and eliminates a
                     potential race condition where the sender deletes
                     the cached link-layer address prior to receiving a
                     response to a previous solicitation.

      Future versions of this protocol may define new option types.
      Receivers MUST silently ignore any options they do not recognize
      and continue processing the message.
*/

const ROUTER_FLAG_POS: u8 = 0;
const SOLICITED_FLAG_POS: u8 = 1;
const OVERRIDE_FLAG_POS: u8 = 2;

#[derive(Debug)]
#[repr(C)]
pub struct Icmpv6NeighborAdvertisement<T>
    where
        T: Ipv6VarHeader,
{
    icmp_neighbor: Icmpv6Neighbor<T>,
    router_flag: bool,
    solicitated_flag: bool,
    override_flag: bool,
    _parent: PhantomData<T>,
}

impl<T> Default for Icmpv6NeighborAdvertisement<T>
    where
        T: Ipv6VarHeader,
{
    fn default() -> Icmpv6NeighborAdvertisement<T> {
        Icmpv6NeighborAdvertisement {
            icmp_neighbor: Icmpv6Neighbor {
                icmp: Icmpv6Header {
                    msg_type: IcmpMessageType::NeighborSolicitation as u8,
                    ..Default::default()
                },
                ..Default::default()
            },
            router_flag: false,
            solicitated_flag: false,
            override_flag: false,
            _parent: PhantomData,
        }
    }
}

impl<T> fmt::Display for Icmpv6NeighborAdvertisement<T>
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

impl<T> EndOffset for Icmpv6NeighborAdvertisement<T>
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
    fn payload_size(&self, hint: usize) -> usize {
        // There is no payload size in the ICMPv6 header
        hint - self.offset()
    }

    #[inline]
    fn check_correct(&self, _prev: &T) -> bool {
        true
    }
}

impl<T> Icmpv6NeighborAdvertisement<T>
    where
        T: Ipv6VarHeader,
{
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn msg_type(&self) -> Option<IcmpMessageType> {
        self.icmp_neighbor.icmp.msg_type()
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
        u32::from_be(self.icmp_neighbor.options.options)
    }

    #[inline]
    pub fn router_flag(&self) -> bool {
        get_bit(self.reserved_flags().to_be_bytes()[3], ROUTER_FLAG_POS)
    }

    #[inline]
    pub fn solicitated_flag(&self) -> bool {
        get_bit(self.reserved_flags().to_be_bytes()[3], SOLICITED_FLAG_POS)
    }

    #[inline]
    pub fn override_flag(&self) -> bool {
       get_bit(self.reserved_flags().to_be_bytes()[3], OVERRIDE_FLAG_POS)
    }

    #[inline]
    pub fn target_addr(&self) -> Ipv6Addr {
        self.icmp_neighbor.target_addr()
    }
}
