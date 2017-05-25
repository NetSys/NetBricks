use allocators::*;
use common::*;
use native::zcsi::*;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use super::PortStats;
use super::super::{PacketTx, PacketRx};

type CallbackFunc<T> = fn(cookie: &T, pkts: &mut [*mut MBuf]) -> Result<u32>;

pub struct CallbackPort<T> {
    cb_rx: CallbackFunc<T>,
    cb_tx: CallbackFunc<T>,
    stats_rx: Arc<CacheAligned<PortStats>>,
    stats_tx: Arc<CacheAligned<PortStats>>,
}

pub struct CallbackQueue<T> {
    cookie: Arc<T>,
    cb_rx: CallbackFunc<T>,
    cb_tx: CallbackFunc<T>,
    stats_rx: Arc<CacheAligned<PortStats>>,
    stats_tx: Arc<CacheAligned<PortStats>>,
}

impl<T: Clone> Clone for CallbackQueue<T> {
    fn clone(&self) -> CallbackQueue<T> {
        CallbackQueue {
            cookie: self.cookie.clone(),
            cb_rx: self.cb_rx,
            cb_tx: self.cb_tx,
            stats_rx: self.stats_rx.clone(),
            stats_tx: self.stats_tx.clone(),
        }
    }
}

impl<T> fmt::Display for CallbackQueue<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "callback queue")
    }
}

impl<T: Send + Sync> PacketRx for CallbackQueue<T> {
    #[inline]
    fn recv(&self, pkts: &mut [*mut MBuf]) -> Result<u32> {
        let ret = (self.cb_rx)(&self.cookie, pkts);
        if let Ok(cnt) = ret {
            let update = self.stats_rx.stats.load(Ordering::Relaxed) + cnt as usize;
            self.stats_rx.stats.store(update, Ordering::Relaxed);
        }
        return ret;
    }
}

impl<T: Send + Sync> PacketTx for CallbackQueue<T> {
    #[inline]
    fn send(&self, pkts: &mut [*mut MBuf]) -> Result<u32> {
        let ret = (self.cb_tx)(&self.cookie, pkts);
        if let Ok(cnt) = ret {
            let update = self.stats_tx.stats.load(Ordering::Relaxed) + cnt as usize;
            self.stats_tx.stats.store(update, Ordering::Relaxed);
        }
        return ret;
    }
}

impl<T> CallbackPort<T> {
    pub fn new(_queues: i32, cb_rx: CallbackFunc<T>, cb_tx: CallbackFunc<T>) -> Result<Arc<CallbackPort<T>>> {
        Ok(Arc::new(CallbackPort {
            cb_rx: cb_rx,
            cb_tx: cb_tx,
            stats_rx: Arc::new(PortStats::new()),
            stats_tx: Arc::new(PortStats::new()),
        }))
    }

    pub fn new_callback_queue(&self, cookie: T) -> Result<CacheAligned<CallbackQueue<T>>> {
        Ok(CacheAligned::allocate(CallbackQueue {
            cookie: Arc::new(cookie),
            cb_rx: self.cb_rx,
            cb_tx: self.cb_tx,
            stats_rx: self.stats_rx.clone(),
            stats_tx: self.stats_tx.clone(),
        }))
    }

    /// Get stats for an RX/TX queue pair.
    pub fn stats(&self) -> (usize, usize) {
        (self.stats_rx.stats.load(Ordering::Relaxed), self.stats_tx.stats.load(Ordering::Relaxed))
    }
}
