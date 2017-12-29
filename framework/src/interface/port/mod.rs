pub use self::phy_port::*;
pub use self::virt_port::*;
use allocators::*;
use common::*;
use interface::{PacketRx, PacketTx};
use native::zcsi::MBuf;
use std::sync::atomic::AtomicUsize;
mod phy_port;
mod virt_port;

/// Statistics for PMD port.
struct PortStats {
    pub stats: AtomicUsize,
}

impl PortStats {
    pub fn new() -> CacheAligned<PortStats> {
        CacheAligned::allocate(PortStats {
            stats: AtomicUsize::new(0),
        })
    }
}

impl<T: PacketRx> PacketRx for CacheAligned<T> {
    #[inline]
    fn recv(&self, pkts: &mut [*mut MBuf]) -> Result<u32> {
        T::recv(&*self, pkts)
    }
}

impl<T: PacketTx> PacketTx for CacheAligned<T> {
    #[inline]
    fn send(&self, pkts: &mut [*mut MBuf]) -> Result<u32> {
        T::send(&*self, pkts)
    }
}
