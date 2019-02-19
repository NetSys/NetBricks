use super::{IcmpMessageType, Icmpv6Header};
use headers::ip::v6::icmp::neighbor_options::*;
use headers::mac::MacAddress;
use headers::{CalcChecksums, EndOffset, Ipv6VarHeader};
use num::FromPrimitive;
use std::collections::HashMap;
use std::default::Default;
use std::fmt;
use std::marker::PhantomData;
use std::slice;
use utils::*;
/*
  ICMPv6 messages are contained in IPv6 packets. The IPv6 packet contains an IPv6 header followed by the
  payload which contains the ICMPv6 message.

  From (https://tools.ietf.org/html/rfc4861)
  The ICMPv6 Router Advertisement Messages have the following general format:

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
                     router list.  The Router Lifetime applies only to
                     the router's usefulness as a default router; it
                     does not apply to information contained in other
                     message fields or options.  Options that need time
                     limits for their information include their own
                     lifetime fields.

      Reachable Time 32-bit unsigned integer.  The time, in
                     milliseconds, that a node assumes a neighbor is
                     reachable after having received a reachability
                     confirmation.  Used by the Neighbor Unreachability
                     Detection algorithm (see Section 7.3).  A value of
                     zero means unspecified (by this router).

      Retrans Timer  32-bit unsigned integer.  The time, in
                     milliseconds, between retransmitted Neighbor
                     Solicitation messages.  Used by address resolution
                     and the Neighbor Unreachability Detection algorithm
                     (see Sections 7.2 and 7.3).  A value of zero means
                     unspecified (by this router).

   Possible options:

      Source link-layer address
                     The link-layer address of the interface from which
                     the Router Advertisement is sent.  Only used on
                     link layers that have addresses.  A router MAY omit
                     this option in order to enable inbound load sharing
                     across multiple link-layer addresses.

      MTU            SHOULD be sent on links that have a variable MTU
                     (as specified in the document that describes how to
                     run IP over the particular link type).  MAY be sent
                     on other links.

      Prefix Information
                     These options specify the prefixes that are on-link
                     and/or are used for stateless address
                     autoconfiguration.  A router SHOULD include all its
                     on-link prefixes (except the link-local prefix) so
                     that multihomed hosts have complete prefix
                     information about on-link destinations for the
                     links to which they attach.  If complete
                     information is lacking, a host with multiple
                     interfaces may not be able to choose the correct
                     outgoing interface when sending traffic to its
                     neighbors.
      Future versions of this protocol may define new option types.
      Receivers MUST silently ignore any options they do not recognize
      and continue processing the message.

*/

const MANAGED_CFG_ADDR_POS: u8 = 0;
const OTHER_CFG_POS: u8 = 1;

#[derive(Debug)]
#[repr(C, packed)]
pub struct Icmpv6RouterAdvertisement<T>
where
    T: Ipv6VarHeader,
{
    icmp: Icmpv6Header<T>,
    current_hop_limit: u8,
    reserved_flags: u8,
    router_lifetime: u16,
    reachable_time: u32,
    retrans_timer: u32,
    options: HashMap<Icmpv6OptionType, Icmpv6Option>,
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
            options: HashMap::new(),
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
            "msg_type: {} code: {} checksum: {}, current_hop_limit {}, reserved_flags {}, router_lifetime {}, reachable_time {}, retrans_timers {}",
            self.msg_type().unwrap(),
            self.code(),
            self.checksum(),
            self.current_hop_limit(),
            self.reserved_flags(),
            self.router_lifetime(),
            self.reachable_time(),
            self.retrans_timer(),
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
        // ICMPv6 Header for Router Advertisement (Type + Code + Checksum)
        // is always 8 bytes: (8 + 8 + 16) / 8 = 4
        // Options are a variable length and will be manually parsed
        16
    }

    #[inline]
    fn size() -> usize {
        // ICMPv6 Header is always 8 bytes so size = offset
        16
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
}

impl<T> IPv6Optionable for Icmpv6RouterAdvertisement<T>
where
    T: Ipv6VarHeader,
{
    fn parse_options(&self, payload_len: u16) -> HashMap<Icmpv6OptionType, Icmpv6Option> {
        let mut options_map = HashMap::new();
        unsafe {
            let self_as_u8 = (self as *const Self) as *const u8;
            let mut payload_offset = self.offset(); //track to make sure we don't go beyond v6 payload size

            while payload_offset < (payload_len as usize) {
                // start at beginning of the options
                // the second byte is the header length in 8-octet unit excluding the first 8 octets
                let seek_to = self_as_u8.offset(payload_offset as isize);

                //lets get the first two indices which should be option_type and option_length fields
                let option_meta = slice::from_raw_parts(seek_to, 2);

                //Parse the option type field first then the option_length
                let option_type: Option<Icmpv6OptionType> = FromPrimitive::from_u8(option_meta[0]);
                let option_length = option_meta[1];
                let option_length_octets = option_length * 8;

                //Make sure the option is valid defined by the RFC. Note: a router can insert its own
                //options defined outside of the spec.
                if option_type.is_some() {
                    //option_value_length is always total number of octets - 2(option type and length are always 2)
                    let option_value_length = (option_length_octets - 2) as isize;

                    let value_seek_to = self_as_u8.offset((payload_offset + 2) as isize);
                    let option_value =
                        slice::from_raw_parts(value_seek_to, option_value_length as usize);

                    match option_type.unwrap() {
                        Icmpv6OptionType::SourceLinkLayerAddress => options_map.insert(
                            option_type.unwrap(),
                            Icmpv6Option::SourceLinkLayerAddress(MacAddress::new(
                                option_value[0],
                                option_value[1],
                                option_value[2],
                                option_value[3],
                                option_value[4],
                                option_value[5],
                            )),
                        ),
                        _ => None,
                    };
                }

                payload_offset += (option_length_octets) as usize;
            }
        }
        options_map
    }
}
