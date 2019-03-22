use packets::Packet;

pub mod v6;

/// IP packet marker trait
pub trait IpPacket: Packet {}
