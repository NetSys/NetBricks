use packets::icmp::v6::{Icmpv6, Icmpv6Packet, Icmpv6Payload, Icmpv6Type, Icmpv6Types, NdpPayload};
use packets::ip::v6::Ipv6Packet;
use std::fmt;
use std::net::Ipv6Addr;

/*  From https://tools.ietf.org/html/rfc4861#section-4.4
    Neighbor Advertisement Message Format

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

    R               Router flag.  When set, the R-bit indicates that
                    the sender is a router.  The R-bit is used by
                    Neighbor Unreachability Detection to detect a
                    router that changes to a host.

    S               Solicited flag.  When set, the S-bit indicates that
                    the advertisement was sent in response to a
                    Neighbor Solicitation from the Destination address.
                    The S-bit is used as a reachability confirmation
                    for Neighbor Unreachability Detection.  It MUST NOT
                    be set in multicast advertisements or in
                    unsolicited unicast advertisements.

    O               Override flag.  When set, the O-bit indicates that
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

    Reserved        29-bit unused field.  It MUST be initialized to
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
*/

const R_FLAG: u8 = 0b1000_0000;
const S_FLAG: u8 = 0b0100_0000;
const O_FLAG: u8 = 0b0010_0000;

/// NDP neighbor advertisement message
#[derive(Debug)]
#[repr(C)]
pub struct NeighborAdvertisement {
    flags: u8,
    reserved1: u8,
    reserved2: u16,
    target_addr: Ipv6Addr,
}

impl Default for NeighborAdvertisement {
    fn default() -> NeighborAdvertisement {
        NeighborAdvertisement {
            flags: 0,
            reserved1: 0,
            reserved2: 0,
            target_addr: Ipv6Addr::UNSPECIFIED,
        }
    }
}

impl Icmpv6Payload for NeighborAdvertisement {
    #[inline]
    fn msg_type() -> Icmpv6Type {
        Icmpv6Types::NeighborAdvertisement
    }
}

impl NdpPayload for NeighborAdvertisement {}

/// NDP neighbor advertisement packet
impl<E: Ipv6Packet> Icmpv6<E, NeighborAdvertisement> {
    #[inline]
    pub fn router(&self) -> bool {
        self.payload().flags & R_FLAG != 0
    }

    #[inline]
    pub fn set_router(&mut self) {
        self.payload_mut().flags |= R_FLAG;
    }

    #[inline]
    pub fn unset_router(&mut self) {
        self.payload_mut().flags &= !R_FLAG;
    }

    #[inline]
    pub fn solicited(&self) -> bool {
        self.payload().flags & S_FLAG != 0
    }

    #[inline]
    pub fn set_solicited(&mut self) {
        self.payload_mut().flags |= S_FLAG;
    }

    #[inline]
    pub fn unset_solicited(&mut self) {
        self.payload_mut().flags &= !S_FLAG;
    }

    #[inline]
    pub fn r#override(&self) -> bool {
        self.payload().flags & O_FLAG != 0
    }

    #[inline]
    pub fn set_override(&mut self) {
        self.payload_mut().flags |= O_FLAG;
    }

    #[inline]
    pub fn unset_override(&mut self) {
        self.payload_mut().flags &= !O_FLAG;
    }

    #[inline]
    pub fn target_addr(&self) -> Ipv6Addr {
        self.payload().target_addr
    }

    #[inline]
    pub fn set_target_addr(&mut self, target_addr: Ipv6Addr) {
        self.payload_mut().target_addr = target_addr
    }
}

impl<E: Ipv6Packet> fmt::Display for Icmpv6<E, NeighborAdvertisement> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "type: {}, code: {}, checksum: 0x{:04x}, router: {}, solicited: {}, override: {}, target address: {}",
            self.msg_type(),
            self.code(),
            self.checksum(),
            self.router(),
            self.solicited(),
            self.r#override(),
            self.target_addr()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use packets::Fixed;

    #[test]
    fn size_of_neighbor_advertisement() {
        assert_eq!(20, NeighborAdvertisement::size());
    }
}
