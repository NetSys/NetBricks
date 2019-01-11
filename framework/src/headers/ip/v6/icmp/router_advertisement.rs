use super::{IcmpMessageType, Icmpv6Header};
use headers::{CalcChecksums, EndOffset, Ipv6VarHeader};
use std::default::Default;
use std::fmt;
use std::marker::PhantomData;
use utils::*;

/*
  ICMPv6 messages are contained in IPv6 packets. The IPv6 packet contains an IPv6 header followed by the
  payload which contains the ICMPv6 message.

  From (https://tools.ietf.org/html/rfc4861)
  The ICMPv6 messages have the following general format:

      0                   1                   2                   3
      0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
     +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
     |     Type      |     Code      |          Checksum             |
     +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
     | Cur Hop Limit |M|O|  Reserved |       Router Lifetime         |
     +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
     |                         Reachable Time                        |
     +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
     |                          Retrans Timer                        |
     +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
     |   Options ...
     +-+-+-+-+-+-+-+-+-+-+-+-

  ICMP Fields:

      Type           134

      Code           0

      Checksum       The ICMP checksum.  See [ICMPv6].

      Cur Hop Limit  8-bit unsigned integer.  The default value that
                     should be placed in the Hop Count field of the IP
                     header for outgoing IP packets.  A value of zero
                     means unspecified (by this router).

      M              1-bit "Managed address configuration" flag.  When
                     set, it indicates that addresses are available via
                     Dynamic Host Configuration Protocol [DHCPv6].

                     If the M flag is set, the O flag is redundant and
                     can be ignored because DHCPv6 will return all
                     available configuration information.

      O              1-bit "Other configuration" flag.  When set, it
                     indicates that other configuration information is
                     available via DHCPv6.  Examples of such information
                     are DNS-related information or information on other
                     servers within the network.

      Note: If neither M nor O flags are set, this indicates that no
      information is available via DHCPv6.

      Reserved       A 6-bit unused field.  It MUST be initialized to
                     zero by the sender and MUST be ignored by the
                     receiver.

      Router Lifetime
                     16-bit unsigned integer.  The lifetime associated
                     with the default router in units of seconds.  The
                     field can contain values up to 65535 and receivers
                     should handle any value, while the sending rules in
                     Section 6 limit the lifetime to 9000 seconds.  A
                     Lifetime of 0 indicates that the router is not a
                     default router and SHOULD NOT appear on the default
*/

const MANAGED_CFG_ADDR_POS: u8 = 0;
const OTHER_CFG_POS: u8 = 1;

#[derive(Debug)]
#[repr(C, packed)]
pub struct IcmpOptions {
    options: u32,
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct Icmpv6RouterAdvertisement<T>
where
    T: Ipv6VarHeader,
{
    pub icmp: Icmpv6Header<T>,
    current_hop_limit: u8,
    reserved_flags: u8,
    router_lifetime: u16,
    reachable_time: u32,
    retrans_timer: u32,
    pub options: IcmpOptions,
    _parent: PhantomData<T>,
}

impl<T> Default for Icmpv6RouterAdvertisement<T>
where
    T: Ipv6VarHeader,
{
    fn default() -> Icmpv6RouterAdvertisement<T> {
        Icmpv6RouterAdvertisement {
            icmp: Icmpv6Header {
                msg_type: IcmpMessageType::RouterAdvertisement as u8,
                code: 0,
                checksum: 0,
                ..Default::default()
            },
            current_hop_limit: 0,
            reserved_flags: 0,
            router_lifetime: 0,
            reachable_time: 0,
            retrans_timer: 0,
            options: IcmpOptions { options: 0 },
            _parent: PhantomData,
        }
    }
}

impl<T> fmt::Display for Icmpv6RouterAdvertisement<T>
where
    T: Ipv6VarHeader,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "msg_type: {} code: {} checksum: {}, current_hop_limit {}, reserved_flags {}, router_lifetime {}, reachable_time {}, retrans_timers {}, options {}",
            self.msg_type().unwrap(),
            self.code(),
            self.checksum(),
            self.current_hop_limit(),
            self.reserved_flags(),
            self.router_lifetime(),
            self.reachable_time(),
            self.retrans_timer(),
            self.options()
        )
    }
}

impl<T> EndOffset for Icmpv6RouterAdvertisement<T>
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

impl<T> Icmpv6RouterAdvertisement<T>
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
        self.icmp.code()
    }

    #[inline]
    pub fn checksum(&self) -> u16 {
        self.icmp.checksum()
    }

    #[inline]
    pub fn current_hop_limit(&self) -> u8 {
        self.current_hop_limit
    }

    #[inline]
    pub fn reserved_flags(&self) -> u8 {
        self.reserved_flags
    }

    #[inline]
    pub fn managed_addr_cfg(&self) -> bool {
        get_bit(self.reserved_flags, MANAGED_CFG_ADDR_POS)
    }

    #[inline]
    pub fn other_cfg(&self) -> bool {
        get_bit(self.reserved_flags, OTHER_CFG_POS)
    }

    #[inline]
    pub fn router_lifetime(&self) -> u16 {
        u16::from_be(self.router_lifetime)
    }

    #[inline]
    pub fn reachable_time(&self) -> u32 {
        u32::from_be(self.reachable_time)
    }

    #[inline]
    pub fn retrans_timer(&self) -> u32 {
        u32::from_be(self.retrans_timer)
    }

    #[inline]
    pub fn options(&self) -> u32 {
        u32::from_be(self.options.options)
    }
}
