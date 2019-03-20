use packets::icmp::v6::{Icmpv6, Icmpv6Packet, Icmpv6Payload, NdpOption};

pub mod options;
pub mod router_advert;
pub mod router_solicit;

/// ndp payload marker trait
pub trait NdpPayload: Icmpv6Payload {}

/// common fn all ndp packets share
pub trait NdpPacket<T: NdpPayload>: Icmpv6Packet<T> {
    fn find_option<O: NdpOption>(&self) -> Option<&mut O> {
        unsafe {
            // options are after the fixed part of the payload
            let mut offset = self.payload_offset() + T::size();
            let mut buffer_left = self.payload_len() - T::size();

            while buffer_left > O::size() {
                let [option_type, length] = *(self.get_mut_item::<[u8; 2]>(offset));

                if option_type == O::option_type() {
                    return Some(&mut (*(self.get_mut_item::<O>(offset))))
                } else if length == 0 {
                    return None    // TODO: should we error?
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

impl<T: NdpPayload> NdpPacket<T> for Icmpv6<T> where Icmpv6<T>: Icmpv6Packet<T> {}
