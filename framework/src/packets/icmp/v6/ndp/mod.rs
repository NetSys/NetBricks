mod neighbor_advert;
mod neighbor_solicit;
pub mod options;
mod router_advert;
mod router_solicit;


pub use self::neighbor_advert::*;
pub use self::neighbor_solicit::*;
pub use self::options::*;
pub use self::router_advert::*;
pub use self::router_solicit::*;

use super::{Icmpv6, Icmpv6Packet, Icmpv6Payload};
use crate::packets::ip::v6::Ipv6Packet;
use crate::packets::Packet;
use crate::packets::{buffer};
use packets::icmp::v6::ndp::options::NdpOption;
use packets::icmp::v6::ndp::options::NdpOptionsTrait;
use packets::Fixed;


/// NDP message payload marker
pub trait NdpPayload: Icmpv6Payload {}

/// Common behaviors shared by NDP packets
///
/// NDP packets are also ICMPv6 packets.
pub trait NdpPacket<E: Ipv6Packet, P: NdpPayload>: Icmpv6Packet<E, P> {
    /// Returns an iterator that iterates through the options in the NDP packet
    fn options(&self) -> NdpOptionsIterator;
    fn add_option<T: NdpOptionsTrait>(&self, option: T) ->;
}





impl<E: Ipv6Packet, P: NdpPayload> NdpPacket<E, P> for Icmpv6<E, P>
where
    Icmpv6<E, P>: Icmpv6Packet<E, P>,
{
    fn options(&self) -> NdpOptionsIterator {
        let mbuf = self.mbuf();
        let offset = self.payload_offset() + P::size();
        NdpOptionsIterator::new(mbuf, offset)
    }

    fn add_option<T: NdpOptionsTrait>(&self, option: T) -> T {
        ///how does this get called for the ndp options trait
    }

    /// Fixed packet header marker trait
///
/// Some packet headers are variable in length, such as the IPv6
/// segment routing header. The fixed portion can be statically
/// defined, but the variable portion has to be parsed separately.



    /*  fn set_options(&self, options: Vec<NdpOption> , offset: usize) {
          buffer::alloc(self.mbuf, offset,self.len() + offset);
          for option in options {

              match option {
                  NdpOption::SourceLinkLayerAddress(value) =>
                      println!("{}", value),
                  NdpOption::TargetLinkLayerAddress(value) =>
                      println!("{}", value),
                  NdpOption::PrefixInformation(value) =>
                      println!("{}", value),
                  NdpOption::Mtu(value) =>
                      println!("{}", value),
                  NdpOption::Undefined(_,value) =>
                      println!("{}", value)
              }


          }*/
   // }
}

