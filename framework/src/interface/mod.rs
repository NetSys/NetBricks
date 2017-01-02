pub use self::packet::*;
pub use self::port::*;
pub mod dpdk;
mod port;
mod packet;
use native::zcsi::MBuf;
use common::*;

/// Generic trait for objects that can perform packet I/O.
pub trait PacketRx : Send {
    fn recv(&self, pkts: &mut [*mut MBuf], to_recv: i32) -> Result<u32>;
}

pub trait PacketTx : Send {
    fn send(&self, pkts: &mut [*mut MBuf], to_send: i32) -> Result<u32>;
}
