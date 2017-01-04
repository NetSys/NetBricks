use allocators::*;
use common::*;
use native::zcsi::*;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use super::PortStats;
use super::super::{PacketTx, PacketRx};

pub struct VirtualPort {
    stats_rx: Arc<CacheAligned<PortStats>>,
    stats_tx: Arc<CacheAligned<PortStats>>,
}

#[derive(Clone)]
pub struct VirtualQueue {
    stats_rx: Arc<CacheAligned<PortStats>>,
    stats_tx: Arc<CacheAligned<PortStats>>,
}

impl PacketTx for VirtualQueue {
    #[inline]
    fn send(&self, pkts: &mut [*mut MBuf]) -> Result<u32> {
        let len = pkts.len() as i32;
        let update = self.stats_tx.stats.load(Ordering::Relaxed) + len as usize;
        self.stats_tx.stats.store(update, Ordering::Relaxed);
        unsafe {
            mbuf_free_bulk(pkts.as_mut_ptr(), len);
        }
        Ok(len as u32)
    }
}

impl PacketRx for VirtualQueue {
    /// Send a batch of packets out this PortQueue. Note this method is internal to NetBricks (should not be directly
    /// called).
    #[inline]
    fn recv(&self, pkts: &mut [*mut MBuf]) -> Result<u32> {
        let len = pkts.len() as i32;
        let status = unsafe { mbuf_alloc_bulk(pkts.as_mut_ptr(), 60, len) };
        let alloced = if status == 0 { len } else { 0 };
        let update = self.stats_rx.stats.load(Ordering::Relaxed) + alloced as usize;
        self.stats_rx.stats.store(update, Ordering::Relaxed);
        Ok(alloced as u32)
    }
}

impl VirtualPort {
    pub fn new(_queues: i32) -> Result<Arc<VirtualPort>> {
        Ok(Arc::new(VirtualPort {
            stats_rx: Arc::new(PortStats::new()),
            stats_tx: Arc::new(PortStats::new()),
        }))
    }

    pub fn new_virtual_queue(&self, _queue: i32) -> Result<CacheAligned<VirtualQueue>> {
        Ok(CacheAligned::allocate(VirtualQueue {
            stats_rx: self.stats_rx.clone(),
            stats_tx: self.stats_tx.clone(),
        }))
    }

    /// Get stats for an RX/TX queue pair.
    pub fn stats(&self) -> (usize, usize) {
        (self.stats_rx.stats.load(Ordering::Relaxed), self.stats_tx.stats.load(Ordering::Relaxed))
    }
}
