use allocators::*;
pub use self::phy_port::*;
pub use self::virt_port::*;
use std::sync::atomic::AtomicUsize;
mod phy_port;
mod virt_port;

/// Statistics for PMD port.
struct PortStats {
    pub stats: AtomicUsize,
}

impl PortStats {
    pub fn new() -> CacheAligned<PortStats> {
        CacheAligned::allocate(PortStats { stats: AtomicUsize::new(0) })
    }
}
