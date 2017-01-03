pub use self::packet::*;
pub use self::port::*;
pub mod dpdk;
mod port;
mod packet;
use common::*;
use native::zcsi::MBuf;

/// Generic trait for objects that can receive packets.
pub trait PacketRx: Send {
    fn recv(&self, pkts: &mut [*mut MBuf]) -> Result<u32>;
}

/// Generic trait for objects that can send packets.
pub trait PacketTx: Send {
    fn send(&self, pkts: &mut [*mut MBuf]) -> Result<u32>;
}

pub trait PacketRxTx: PacketRx + PacketTx {}
