use std::fmt;
use std::net::Ipv6Addr;
use packets::ethernet::MacAddr;

const OPT_SOURCE_LINK_LAYER_ADDR: u8 = 1;
const OPT_TARGET_LINK_LAYER_ADDR: u8 = 2;
const OPT_PREFIX_INFORMATION: u8 = 3;
//const OPT_REDIRECTED_HEADER: u8 = 4;
const OPT_MTU: u8 = 5;

/// Neighbor discovery message option
pub trait NdpOption {
    /// Returns the size of the NDP option in bytes
    fn size() -> usize;

    /// Returns the type code of the NDP option
    fn option_type() -> u8;
}

/*  From (https://tools.ietf.org/html/rfc4861#section-4.6.1)
    Source/Target Link-layer Address

    0                   1                   2                   3
    0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |     Type      |    Length     |    Link-Layer Address ...
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

    Type           1 for Source Link-layer Address
                   2 for Target Link-layer Address


    Length         The length of the option (including the type and
                   length fields) in units of 8 octets.  For example,
                   the length for IEEE 802 addresses is 1.

    Link-Layer Address
                   The variable length link-layer address.

                   The content and format of this field (including
                   byte and bit ordering) is expected to be specified
                   in specific documents that describe how IPv6
                   operates over different link layers.
*/

/// Source link-layer address option
#[derive(Debug)]
#[repr(C, packed)]
pub struct SourceLinkLayerAddress {
    option_type: u8,
    length: u8,
    addr: MacAddr
}

impl SourceLinkLayerAddress {
    #[inline]
    pub fn length(&self) -> u8 {
        self.length
    }

    #[inline]
    pub fn addr(&self) -> MacAddr {
        self.addr
    }

    #[inline]
    pub fn set_addr(&mut self, addr: MacAddr) {
        self.addr = addr;
    }
}

impl Default for SourceLinkLayerAddress {
    fn default() -> SourceLinkLayerAddress {
        SourceLinkLayerAddress {
            option_type: OPT_SOURCE_LINK_LAYER_ADDR,
            length: 1,
            addr: Default::default()
        }
    }
}

impl fmt::Display for SourceLinkLayerAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "type: {}, length: {}, addr: {}",
            self.option_type,
            self.length(),
            self.addr()
        )
    }
}

impl NdpOption for SourceLinkLayerAddress {
    #[inline]
    fn size() -> usize {
        8
    }

    #[inline]
    fn option_type() -> u8 {
        OPT_SOURCE_LINK_LAYER_ADDR
    }
}

/// Target link-layer address option
#[derive(Debug)]
#[repr(C, packed)]
pub struct TargetLinkLayerAddress {
    option_type: u8,
    length: u8,
    addr: MacAddr
}

impl TargetLinkLayerAddress {
    #[inline]
    pub fn length(&self) -> u8 {
        self.length
    }

    #[inline]
    pub fn addr(&self) -> MacAddr {
        self.addr
    }

    #[inline]
    pub fn set_addr(&mut self, addr: MacAddr) {
        self.addr = addr;
    }
}

impl Default for TargetLinkLayerAddress {
    fn default() -> TargetLinkLayerAddress {
        TargetLinkLayerAddress {
            option_type: OPT_TARGET_LINK_LAYER_ADDR,
            length: 1,
            addr: Default::default()
        }
    }
}

impl fmt::Display for TargetLinkLayerAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "type: {}, length: {}, addr: {}",
            self.option_type,
            self.length(),
            self.addr()
        )
    }
}

impl NdpOption for TargetLinkLayerAddress {
    #[inline]
    fn size() -> usize {
        8
    }

    #[inline]
    fn option_type() -> u8 {
        OPT_TARGET_LINK_LAYER_ADDR
    }
}

/*  From (https://tools.ietf.org/html/rfc4861#section-4.6.2)
    Prefix Information

    0                   1                   2                   3
    0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |     Type      |    Length     | Prefix Length |L|A| Reserved1 |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |                         Valid Lifetime                        |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |                       Preferred Lifetime                      |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |                           Reserved2                           |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |                                                               |
    +                                                               +
    |                                                               |
    +                            Prefix                             +
    |                                                               |
    +                                                               +
    |                                                               |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

    Type           3

    Length         4

    Prefix Length  8-bit unsigned integer.  The number of leading bits
                   in the Prefix that are valid.  The value ranges
                   from 0 to 128.  The prefix length field provides
                   necessary information for on-link determination
                   (when combined with the L flag in the prefix
                   information option).  It also assists with address
                   autoconfiguration as specified in [ADDRCONF], for
                   which there may be more restrictions on the prefix
                   length.

    L              1-bit on-link flag.  When set, indicates that this
                   prefix can be used for on-link determination.  When
                   not set the advertisement makes no statement about
                   on-link or off-link properties of the prefix.  In
                   other words, if the L flag is not set a host MUST
                   NOT conclude that an address derived from the
                   prefix is off-link.  That is, it MUST NOT update a
                   previous indication that the address is on-link.

    A              1-bit autonomous address-configuration flag.  When
                   set indicates that this prefix can be used for
                   stateless address configuration as specified in
                   [ADDRCONF].

    Reserved1      6-bit unused field.  It MUST be initialized to zero
                   by the sender and MUST be ignored by the receiver.

    Valid Lifetime
                   32-bit unsigned integer.  The length of time in
                   seconds (relative to the time the packet is sent)
                   that the prefix is valid for the purpose of on-link
                   determination.  A value of all one bits
                   (0xffffffff) represents infinity.  The Valid
                   Lifetime is also used by [ADDRCONF].

    Preferred Lifetime
                   32-bit unsigned integer.  The length of time in
                   seconds (relative to the time the packet is sent)
                   that addresses generated from the prefix via
                   stateless address autoconfiguration remain
                   preferred [ADDRCONF].  A value of all one bits
                   (0xffffffff) represents infinity.  See [ADDRCONF].
                   Note that the value of this field MUST NOT exceed
                   the Valid Lifetime field to avoid preferring
                   addresses that are no longer valid.

    Reserved2      This field is unused.  It MUST be initialized to
                   zero by the sender and MUST be ignored by the
                   receiver.

    Prefix         An IP address or a prefix of an IP address.  The
                   Prefix Length field contains the number of valid
                   leading bits in the prefix.  The bits in the prefix
                   after the prefix length are reserved and MUST be
                   initialized to zero by the sender and ignored by
                   the receiver.  A router SHOULD NOT send a prefix
                   option for the link-local prefix and a host SHOULD
                   ignore such a prefix option.
*/

/// Prefix information option
#[derive(Debug)]
#[repr(C)]
pub struct PrefixInformation {
    option_type: u8,
    length: u8,
    prefix_length: u8,
    flags: u8,
    valid_lifetime: u32,
    preferred_lifetime: u32,
    reserved: u32,
    prefix: Ipv6Addr
}

impl PrefixInformation {
    #[inline]
    pub fn length(&self) -> u8 {
        self.length
    }

    #[inline]
    pub fn prefix_length(&self) -> u8 {
        self.prefix_length
    }

    #[inline]
    pub fn set_prefix_length(&mut self, prefix_length: u8) {
        self.prefix_length = prefix_length
    }

    #[inline]
    pub fn on_link(&self) -> bool {
        self.flags & 0x80 > 0
    }

    #[inline]
    pub fn set_on_link(&mut self) {
        self.flags |= 0x80;
    }

    #[inline]
    pub fn unset_on_link(&mut self) {
        self.flags &= 0x7f;
    }

    #[inline]
    pub fn autonomous(&self) -> bool {
        self.flags & 0x40 > 0
    }

    #[inline]
    pub fn set_autonomous(&mut self) {
        self.flags |= 0x40;
    }

    #[inline]
    pub fn unset_autonomous(&mut self) {
        self.flags &= 0xbf;
    }

    #[inline]
    pub fn valid_lifetime(&self) -> u32 {
        u32::from_be(self.valid_lifetime)
    }

    #[inline]
    pub fn set_valid_lifetime(&mut self, valid_lifetime: u32) {
        self.valid_lifetime = u32::to_be(valid_lifetime);
    }

    #[inline]
    pub fn preferred_lifetime(&self) -> u32 {
        u32::from_be(self.preferred_lifetime)
    }

    #[inline]
    pub fn set_preferred_lifetime(&mut self, preferred_lifetime: u32) {
        self.preferred_lifetime = u32::to_be(preferred_lifetime);
    }

    #[inline]
    pub fn prefix(&self) -> Ipv6Addr {
        self.prefix
    }

    #[inline]
    pub fn set_prefix(&mut self, prefix: Ipv6Addr) {
        self.prefix = prefix;
    }
}

impl Default for PrefixInformation {
    fn default() -> PrefixInformation {
        PrefixInformation {
            option_type: OPT_PREFIX_INFORMATION,
            length: 4,
            prefix_length: 0,
            flags: 0,
            valid_lifetime: 0,
            preferred_lifetime: 0,
            reserved: 0,
            prefix: Ipv6Addr::UNSPECIFIED
        }
    }
}

impl fmt::Display for PrefixInformation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "type: {}, length: {}, prefix_length: {}, on_link: {}, autonomous: {}, valid_lifetime: {}, preferred_lifetime: {}, prefix: {}",
            self.option_type,
            self.length(),
            self.prefix_length(),
            self.on_link(),
            self.autonomous(),
            self.valid_lifetime(),
            self.preferred_lifetime(),
            self.prefix()
        )
    }
}

impl NdpOption for PrefixInformation {
    fn size() -> usize {
        32
    }

    fn option_type() -> u8 {
        OPT_PREFIX_INFORMATION
    }
}

/*  From (https://tools.ietf.org/html/rfc4861#section-4.6.4)
    MTU

    0                   1                   2                   3
    0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |     Type      |    Length     |           Reserved            |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |                              MTU                              |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

    Type           5

    Length         1

    Reserved       This field is unused.  It MUST be initialized to
                   zero by the sender and MUST be ignored by the
                   receiver.

    MTU            32-bit unsigned integer.  The recommended MTU for
                   the link.
*/

/// Maximum transmission unit option
#[derive(Debug)]
#[repr(C, packed)]
pub struct Mtu {
    option_type: u8,
    length: u8,
    reserved: u16,
    mtu: u32
}

impl Mtu {
    pub fn length(&self) -> u8 {
        self.length
    }

    pub fn mtu(&self) -> u32 {
        u32::from_be(self.mtu)
    }

    pub fn set_mtu(&mut self, mtu: u32) {
        self.mtu = u32::to_be(mtu);
    }
}

impl Default for Mtu {
    fn default() -> Mtu {
        Mtu {
            option_type: OPT_MTU,
            length: 1,
            reserved: 0,
            mtu: 0
        }
    }
}

impl fmt::Display for Mtu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "type: {}, length: {}, mtu: {}",
            self.option_type,
            self.length(),
            self.mtu()
        )
    }
}

impl NdpOption for Mtu {
    fn size() -> usize {
        8
    }

    fn option_type() -> u8 {
        OPT_MTU
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use packets::MacAddr;

    #[test]
    fn source_link_layer_address() {
        let mut opt: SourceLinkLayerAddress = Default::default();

        assert_eq!(1, opt.length());

        let addr = MacAddr::new(1, 1, 1, 1, 1, 1);
        opt.set_addr(addr);
        assert_eq!(addr, opt.addr());
    }

    #[test]
    fn target_link_layer_address() {
        let mut opt: TargetLinkLayerAddress = Default::default();

        assert_eq!(1, opt.length());

        let addr = MacAddr::new(1, 1, 1, 1, 1, 1);
        opt.set_addr(addr);
        assert_eq!(addr, opt.addr());
    }

    #[test]
    fn prefix_information() {
        let mut opt: PrefixInformation = Default::default();

        assert_eq!(4, opt.length());

        let prefix = Ipv6Addr::new(1, 1, 1, 1, 1, 1, 1, 1);

        opt.set_prefix_length(7);
        opt.set_on_link();
        opt.set_autonomous();
        opt.set_valid_lifetime(1000);
        opt.set_preferred_lifetime(2000);
        opt.set_prefix(prefix);

        assert_eq!(7, opt.prefix_length());
        assert!(opt.on_link());
        assert!(opt.autonomous());
        assert_eq!(1000, opt.valid_lifetime());
        assert_eq!(2000, opt.preferred_lifetime());
        assert_eq!(prefix, opt.prefix());
    }

    #[test]
    fn mtu() {
        let mut opt: Mtu = Default::default();

        assert_eq!(1, opt.length());

        opt.set_mtu(1500);
        assert_eq!(1500, opt.mtu());
    }
}
