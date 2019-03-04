use super::{IcmpMessageType, Icmpv6Header};
use headers::ip::v6::icmp::neighbor_options::*;
use headers::mac::*;
use headers::{CalcChecksums, EndOffset, Ipv6VarHeader};
use num::FromPrimitive;
use std::collections::HashMap;
use std::default::Default;
use std::fmt;
use std::marker::PhantomData;
use std::slice;
use std::net::Ipv6Addr;
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
pub struct NDPOpt {
    pub opt_type: u8,
    pub opt_len: u8,
    pub opt_data: u8
}

impl fmt::Display for NDPOpt
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "opt_type: {} opt_len: {} opt_data: {}",
            self.opt_type,
            self.opt_len,
            self.opt_data,
        )
    }
}

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
    options: u8,
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
            options: 0,
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

#[derive(Debug)]
#[repr(C, packed)]
pub struct SourceLinkLayerAddress {
    pub addr: MacAddress,
}

impl fmt::Display for SourceLinkLayerAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.addr
        )
    }
}

/**
opt_start: a pointer to where the udp options begin
payload_length: the overall length of the payload we cannot scan past
*/
pub fn scan_opt(opt_start: *const u8, offset: usize, payload_length: u16, opt_type: u8) -> Option<*const SourceLinkLayerAddress> {
    unsafe {
        // first thing is the option type
        // second thing is the option length
        let mut payload_offset = offset;
        while (payload_offset as u16) < payload_length {
            println!("scan_opt offset={}, length={}, opt_type={}", payload_offset, payload_length, opt_type);
            let cur_type = *(opt_start.offset(payload_offset as isize));
            let cur_size = *(opt_start.offset((payload_offset + 1) as isize));

            println!("scan_opt cur_type={}, cur_size={}", cur_type, cur_size);

            if cur_size == 0 {
                println!("WHY IS cur_size 0!!!");
                return None;
            } else if cur_type != opt_type {
                // the current type does not match the requested type, so skip to next
                payload_offset = payload_offset + (cur_size as usize);
                println!("Skipping to next offset {}", payload_offset);
            } else {
                // we have a winner!
                println!("WE HAVE A WINNER!!!");
                let cur_start = opt_start.offset((payload_offset + 2) as isize);
                let found_opt = cur_start as *const SourceLinkLayerAddress;
                return Some(found_opt);
            }
        }
        None
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct MtuOption {
    opt_type: u8,
    opt_len: u8,
    reserved: u16,
    mtu: u32,
}


pub fn scan_opt2(opt_start: *const u8, offset: usize, payload_length: u16, opt_type: u8) -> Option<*const MtuOption> {
    unsafe {
        // first thing is the option type
        // second thing is the option length
        let mut payload_offset = offset;
        while (payload_offset as u16) < payload_length {
            println!("scan_opt offset={}, length={}, opt_type={}", payload_offset, payload_length, opt_type);
            let seek_to = opt_start.offset(payload_offset as isize);
            let option_meta = slice::from_raw_parts(seek_to, 2);
            //let cur_type = *(opt_start.offset(payload_offset as isize));
            let cur_type = option_meta[0];
            //let cur_size = *(opt_start.offset((payload_offset + 1) as isize));
            let cur_size = option_meta[1] * 8;

            println!("scan_opt cur_type={}, cur_size={}", cur_type, cur_size);

            if cur_size == 0 {
                println!("WHY IS cur_size 0!!!");
                return None;
            } else if cur_type != opt_type {
                // the current type does not match the requested type, so skip to next
                payload_offset = payload_offset + (cur_size as usize);
                println!("Skipping to next offset {}", payload_offset);
            } else {
                // we have a winner!
                println!("WE HAVE A WINNER!!!");
                let cur_start = opt_start.offset((payload_offset) as isize);
                let found_opt = cur_start as *const MtuOption;
                return Some(found_opt);
            }
        }
        None
    }
}

impl<T> Icmpv6RouterAdvertisement<T>
where
    T: Ipv6VarHeader,
{
    pub fn opt(&self, opt_type: u8, payload_length: u16) -> Option<*const SourceLinkLayerAddress> {
        unsafe {
            //println!("SOURCE LINK LAYER {}", op);
            let self_as_u8 = (self as *const Self) as *const u8;
            let mut payload_offset = self.offset();
            let opt_start = self_as_u8.offset(payload_offset as isize);

            let ott = self_as_u8.offset(payload_offset as isize);
            let ot = *ott;
            println!("opt ot={}", ot);

            let r = scan_opt2(self_as_u8, payload_offset, payload_length, 5);
            match r {
                Some(sll) => {
                    let x = (*sll).mtu;
                    println!("SOURCE PREFIX INFO {}", x);
                },
                None => {
                    println!("NOT FOUND!!!!!");
                }
            }
            return None;
        }
    }

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

//    #[inline]
//    pub fn source_link_layer_address(
//        &self,
//        options: HashMap<NDPOptionType, NDPOption>,
//    ) -> Option<MacAddress> {
//        let source_link_layer = options.get(&NDPOptionType::SourceLinkLayerAddress).unwrap();
//        let source_link_option = match source_link_layer {
//            NDPOption::SourceLinkLayerAddress(value) => Some(*value),
//            _ => None,
//        };
//        source_link_option
//    }
}

//impl<T> NDPOptionParser for Icmpv6RouterAdvertisement<T>
//where
//    T: Ipv6VarHeader,
//{
//    fn parse_options(&self, payload_len: u16) -> HashMap<NDPOptionType, NDPOption> {
//        let mut options_map = HashMap::new();
//        unsafe {
//            let self_as_u8 = (self as *const Self) as *const u8;
//            let mut payload_offset = self.offset(); //track to make sure we don't go beyond v6 payload size
//
//            while payload_offset < (payload_len as usize) {
//                // start at beginning of the options
//                // the second byte is the header length in 8-octet unit excluding the first 8 octets
//                let seek_to = self_as_u8.offset(payload_offset as isize);
//
//                // lets get the first two indices which should be option_type and option_length fields
//                let option_meta = slice::from_raw_parts(seek_to, 2);
//
//                // Parse the option type field first then the option_length
//                let option_type: Option<NDPOptionType> = FromPrimitive::from_u8(option_meta[0]);
//                let option_length = option_meta[1];
//                let option_length_octets = option_length * 8;
//
//                // option_value_length is always total number of octets - 2(option type and length are always 2)
//                let option_value_length = (option_length_octets - 2) as isize;
//                let value_seek_to = self_as_u8.offset((payload_offset + 2) as isize);
//                let option_value =
//                    slice::from_raw_parts(value_seek_to, option_value_length as usize);
//
//                // Make sure the option is valid defined by the RFC. Note: a router can insert its own
//                // options defined outside of the spec.
//                match option_type {
//                    // we only care about matching on source link layer address
//                    Some(NDPOptionType::SourceLinkLayerAddress) => {
//                        options_map.insert(
//                            NDPOptionType::SourceLinkLayerAddress,
//                            NDPOption::SourceLinkLayerAddress(MacAddress::new(
//                                option_value[0],
//                                option_value[1],
//                                option_value[2],
//                                option_value[3],
//                                option_value[4],
//                                option_value[5],
//                            )),
//                        );
//                    }
//                    Some(other) => {
//                        // log maybe?
//                    }
//                    None => {
//                        // log loudly as we cannot handle this option type
//                    }
//                }
//
//                payload_offset += (option_length_octets) as usize;
//            }
//        }
//        options_map
//    }
//}
