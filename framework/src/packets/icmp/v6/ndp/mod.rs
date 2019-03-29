use packets::buffer;
use packets::icmp::v6::{Icmpv6, Icmpv6Packet, Icmpv6Payload, NdpOption};
use packets::ip::v6::Ipv6Packet;

pub mod options;
pub mod router_advert;
pub mod router_solicit;

/// NDP message payload marker
pub trait NdpPayload: Icmpv6Payload {}

/// Common behaviors shared by NDP packets
///
/// NDP packets are also ICMPv6 packets.
pub trait NdpPacket<E: Ipv6Packet, P: NdpPayload>: Icmpv6Packet<E, P> {
    /// finds a NDP option in the payload by option type
    fn find_option<O: NdpOption>(&self) -> Option<&mut O> {
        let payload_size = std::mem::size_of::<P>();
        let option_size = std::mem::size_of::<O>();

        unsafe {
            // options are after the fixed part of the payload
            let mut offset = self.payload_offset() + payload_size;
            let mut buffer_left = self.payload_len() - payload_size;

            while buffer_left > option_size {
                let mbuf = self.mbuf();
                let [option_type, length] = *(buffer::read_item::<[u8; 2]>(mbuf, offset).unwrap());

                if option_type == O::option_type() {
                    return Some(&mut (*(buffer::read_item::<O>(mbuf, offset).unwrap())));
                } else if length == 0 {
                    return None; // TODO: should we error?
                } else {
                    let length = (length * 8) as usize;
                    offset += length;
                    buffer_left -= length;
                }
            }

            None
        }
    }
}

impl<E: Ipv6Packet, P: NdpPayload> NdpPacket<E, P> for Icmpv6<E, P> where
    Icmpv6<E, P>: Icmpv6Packet<E, P>
{
}
