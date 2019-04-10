use netbricks::common::Result;
use netbricks::packets::{Ethernet, Packet, RawPacket};

pub fn macswap(packet: RawPacket) -> Result<Ethernet> {
    assert!(packet.refcnt() == 1);
    let mut ethernet = packet.parse::<Ethernet>()?;
    ethernet.swap_addresses();
    Ok(ethernet)
}
