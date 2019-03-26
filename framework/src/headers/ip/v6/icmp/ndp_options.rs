use std::fmt;
use std::slice;

use num::FromPrimitive;

use headers::ip::v6::icmp::ndp::NdpMessageContents;
use headers::mac::MacAddress;

use log::warn;

#[derive(FromPrimitive, Debug, PartialEq, Hash, Eq, Clone, Copy)]
#[repr(u8)]
pub enum NdpOptionType {
    SourceLinkLayerAddress = 1,
    TargetLinkLayerAddress = 2,
    PrefixInformation = 3,
    RedirectHeader = 4,
    MTU = 5,
    Undefined,
}

impl fmt::Display for NdpOptionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            NdpOptionType::SourceLinkLayerAddress => write!(f, "Source Link Layer Address"),
            NdpOptionType::TargetLinkLayerAddress => write!(f, "Target Link Layer Address"),
            NdpOptionType::PrefixInformation => write!(f, "Prefix Information"),
            NdpOptionType::RedirectHeader => write!(f, "Redirect Header"),
            NdpOptionType::MTU => write!(f, "MTU"),
            NdpOptionType::Undefined => write!(f, "Undefined"),
        }
    }
}

pub trait NdpOption {
    fn get_type() -> NdpOptionType;
    // Size will be used to makes sure we dont go beyond the memory we own when doing memory
    // overlay in option_scan. this is equal to the size of the option struct
    fn get_size() -> u8;
}

/**
    MTU NDP Option type

       0                   1                   2                   3
       0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
      +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
      |     Type      |    Length     |           Reserved            |
      +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
      |                              MTU                              |
      +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

   Fields:

      Type           5

      Length         1

      Reserved       This field is unused.  It MUST be initialized to
                     zero by the sender and MUST be ignored by the
                     receiver.

      MTU            32-bit unsigned integer.  The recommended MTU for
                     the link.

   Description
                     The MTU option is used in Router Advertisement
                     messages to ensure that all nodes on a link use the
                     same MTU value in those cases where the link MTU is
                     not well known.

                     This option MUST be silently ignored for other
                     Neighbor Discovery messages.

                     In configurations in which heterogeneous
                     technologies are bridged together, the maximum
                     supported MTU may differ from one segment to
                     another.  If the bridges do not generate ICMP
                     Packet Too Big messages, communicating nodes will
                     be unable to use Path MTU to dynamically determine
                     the appropriate MTU on a per-neighbor basis.  In
                     such cases, routers can be configured to use the
                     MTU option to specify the maximum MTU value that is
                     supported by all segments.
*/
#[derive(Debug)]
#[repr(C, packed)]
pub struct MtuOption {
    option_type: u8,
    option_length: u8,
    reserved: u16,
    mtu: u32,
}

impl MtuOption {
    /// Retrieves the value of the mtu in the option payload in little endian
    pub fn get_mtu(&self) -> u32 {
        u32::from_be(self.mtu)
    }

    /// Gets the reserved value from the option payload in little endian
    pub fn get_reserved(&self) -> u16 {
        u16::from_be(self.reserved)
    }
}

impl NdpOption for MtuOption {
    fn get_type() -> NdpOptionType {
        NdpOptionType::MTU
    }

    fn get_size() -> u8 {
        // option_type + option_length + reserved + mtu
        // (8 + 8 + 16 + 32) / 8 = 6
        6
    }
}

/**
      0                   1                   2                   3
      0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
     +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
     |     Type      |    Length     |    Link-Layer Address ...
     +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

   Fields:

      Type
                     1 for Source Link-layer Address
                     2 for Target Link-layer Address
      Length         The length of the option (including the type and
                     length fields) in units of 8 octets.  For example,
                     the length for IEEE 802 addresses is 1
                     [IPv6-ETHER].

      Link-Layer Address
                     The variable length link-layer address.

                     The content and format of this field (including
                     byte and bit ordering) is expected to be specified
                     in specific documents that describe how IPv6
                     operates over different link layers.  For instance,
                     [IPv6-ETHER].

   Description
                     The Source Link-Layer Address option contains the
                     link-layer address of the sender of the packet.  It
                     is used in the Neighbor Solicitation, Router
                     Solicitation, and Router Advertisement packets.

                     The Target Link-Layer Address option contains the
                     link-layer address of the target.  It is used in
                     Neighbor Advertisement and Redirect packets.

                     These options MUST be silently ignored for other
                     Neighbor Discovery messages.
*/
#[derive(Debug)]
#[repr(C, packed)]
pub struct LinkLayerAddressOption {
    option_type: u8,
    option_length: u8,
    addr: MacAddress,
}

impl NdpOption for LinkLayerAddressOption {
    fn get_type() -> NdpOptionType {
        NdpOptionType::SourceLinkLayerAddress
    }

    fn get_size() -> u8 {
        // option_type + option_length + mac_address
        // (8 + 8 + 48) / 8 = 8
        8
    }
}

/// Provides functions to implementors that allow the implementor to scan their NDP message for
/// NDP options.  The type of options available are specific to the type of NDP Message
/// Ensures that implementors support NdpMessageContents, which further enforces EndOffset + Sized
/// which we require to do parsing
pub trait NdpOptions: NdpMessageContents {
    /// Attempts to lookup the source link layer address from the underlying NDP Message
    fn get_source_link_layer_address_option(&self, payload_length: u16) -> Option<&MacAddress> {
        let opt = self.option_scan::<LinkLayerAddressOption>(payload_length);
        opt.map(|sll| &sll.addr)
    }

    /// Attempts to lookup the mtu option from the underlying NDP Message
    fn get_mtu_option(&self, payload_length: u16) -> Option<u32> {
        let mtu_option = self
            .option_scan::<MtuOption>(payload_length)
            .map(|mtu| mtu.get_mtu());
        mtu_option
    }

    /// Searches through a packet for an option, returns the first it finds or none
    /// The underlying type is determine by implementing the NdpOption trait which returns the NDP Option Type
    fn option_scan<T: NdpOption>(&self, payload_length: u16) -> Option<&T> {
        unsafe {
            // the start of the options we offset from is the start of the ndp message
            let opt_start = (self as *const Self) as *const u8;
            let mut payload_offset = self.offset();
            let requested_type = T::get_type();

            // make sure we do not scan beyond the end of the packet
            while (payload_offset as u16) < payload_length {
                // we want to pull the first two bytes off, option type and option length
                let seek_to = opt_start.add(payload_offset);
                let option_meta = slice::from_raw_parts(seek_to, 2);

                let cur_type_option: Option<NdpOptionType> = FromPrimitive::from_u8(option_meta[0]);

                let cur_type: NdpOptionType = match cur_type_option {
                    Some(cur_type_option) => cur_type_option,
                    None => NdpOptionType::Undefined,
                };

                // second option is the length in octets (8 byte chunks).  So a length of 1 means 8 bytes
                let cur_size = option_meta[1] * 8;

                if cur_size == 0 {
                    // Don't know how we will be here, but it is an error condition as we will
                    // go into an infinite loop and blow the stack if we skip, so exit None
                    warn!("Option length is set to zero for option type {}", cur_type);
                    return None;
                } else if (payload_offset + cur_size as usize) > (payload_length as usize) {
                    // we need to protect ourselves in case the option_length goes past the payload length
                    warn!("Option length exceeds the payload length for option type {} option_length {}", cur_type, cur_size);
                    return None;
                }

                if cur_type != requested_type {
                    // the current type does not match the requested type, so skip to next
                    payload_offset = payload_offset + (cur_size as usize);
                } else {
                    // we have a winner!
                    // advance the pointer to the current offset
                    let cur_start = opt_start.add(payload_offset);

                    // This check makes sure that our struct doesnt exceed the payload length
                    // and overlay memory that we dont own
                    if (payload_offset + T::get_size() as usize) > (payload_length as usize) {
                        return None;
                    }

                    let found_opt = cur_start as *const T;
                    return Some(&(*found_opt));
                }
            }
            None
        }
    }
}
