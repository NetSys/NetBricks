use super::mbuf::MBuf;
use super::interface::Result;
use super::interface::ZCSIError;
use super::super::headers::MacAddress;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

// External DPDK calls
#[link(name = "zcsi")]
extern "C" {
    fn init_pmd_port(port: i32,
                     rxqs: i32,
                     txqs: i32,
                     rx_cores: *const i32,
                     tx_cores: *const i32,
                     nrxd: i32,
                     ntxd: i32,
                     loopback: i32,
                     tso: i32,
                     csumoffload: i32)
                     -> i32;
    fn free_pmd_port(port: i32) -> i32;
    fn recv_pkts(port: i32, qid: i32, pkts: *mut *mut MBuf, len: i32) -> i32;
    fn send_pkts(port: i32, qid: i32, pkts: *mut *mut MBuf, len: i32) -> i32;
    fn num_pmd_ports() -> i32;
    fn rte_eth_macaddr_get(port: i32, address: *mut MacAddress);
    fn init_bess_eth_ring(ifname: *const u8, core: i32) -> i32;
    fn init_ovs_eth_ring(iface: i32, core: i32) -> i32;
    fn find_port_with_pci_address(pciaddr: *const u8) -> i32;
}

// Make this into an input parameter
const NUM_RXD: i32 = 256;
const NUM_TXD: i32 = 256;


struct PmdStats {
    pub stats: AtomicUsize,
    _pad: [u64; 7],
}

impl PmdStats {
    pub fn new() -> PmdStats {
        PmdStats {
            stats: AtomicUsize::new(0),
            _pad: Default::default(),
        }
    }
}

pub struct PmdPort {
    connected: bool,
    should_close: bool,
    port: i32,
    rxqs: i32,
    txqs: i32,
    stats_rx: Vec<Arc<PmdStats>>,
    stats_tx: Vec<Arc<PmdStats>>,
    _pad: [u64; 4],
}

#[derive(Clone)]
#[repr(C)]
pub struct PortQueue {
    // The Arc cost here should not affect anything, since we are really not doing anything to make it go in and out of
    // scope.
    _pad0: i32,
    pub port: Arc<PmdPort>,
    stats_rx: Arc<PmdStats>,
    stats_tx: Arc<PmdStats>,
    port_id: i32,
    txq: i32,
    rxq: i32,
    _pad1: [u64; 4]
}

impl Drop for PmdPort {
    fn drop(&mut self) {
        if self.connected && self.should_close {
            unsafe {
                free_pmd_port(self.port);
            }
        }
    }
}

/// Represents a single RX/TX queue pair for a port. This is what is needed to send or receive traffic.
impl PortQueue {
    /// Send a batch of packets out this PortQueue. Note this method is internal to ZCSI (should not be directly
    /// called).
    #[inline]
    pub fn send(&mut self, pkts: *mut *mut MBuf, to_send: i32) -> Result<u32> {
        let txq = self.txq;
        self.send_queue(txq, pkts, to_send)
    }

    /// Receive a batch of packets out this PortQueue. Note this method is internal to ZCSI (should not be directly
    /// called).
    #[inline]
    pub fn recv(&self, pkts: *mut *mut MBuf, to_recv: i32) -> Result<u32> {
        let rxq = self.rxq;
        self.recv_queue(rxq, pkts, to_recv)
    }

    #[inline]
    fn send_queue(&mut self, queue: i32, pkts: *mut *mut MBuf, to_send: i32) -> Result<u32> {
        unsafe {
            let sent = send_pkts(self.port_id, queue, pkts, to_send);
            //let update = self.stats_tx.stats.load(Ordering::Relaxed) + sent as usize;
            //self.stats_tx.stats.store(update, Ordering::Relaxed);
            Ok(sent as u32)
        }
    }

    #[inline]
    fn recv_queue(&self, queue: i32, pkts: *mut *mut MBuf, to_recv: i32) -> Result<u32> {
        unsafe {
            let recv = recv_pkts(self.port_id, queue, pkts, to_recv);
            //let update = self.stats_rx.stats.load(Ordering::Relaxed) + recv as usize;
            //self.stats_rx.stats.store(update, Ordering::Relaxed);
            Ok(recv as u32)
        }
    }

    pub fn txq(&self) -> i32 {
        self.txq
    }

    pub fn rxq(&self) -> i32 {
        self.rxq
    }
}

// Utility function to go from Rust bools to C ints. Allowing match bools since this looks nicer to me.
#[cfg_attr(feature = "dev", allow(match_bool))]
fn i32_from_bool(x: bool) -> i32 {
    match x {
        true => 1,
        false => 0,
    }
}

impl PmdPort {
    /// Determine the number of ports in a system.
    pub fn num_pmd_ports() -> i32 {
        unsafe { num_pmd_ports() }
    }

    pub fn find_port_id(pcie: &str) -> i32 {
        unsafe { find_port_with_pci_address(pcie.as_ptr()) }
    }

    pub fn rxqs(&self) -> i32 {
        self.rxqs
    }

    pub fn txqs(&self) -> i32 {
        self.txqs
    }

    pub fn new_queue_pair(port: &Arc<PmdPort>, rxq: i32, txq: i32) -> Result<PortQueue> {
        if rxq > port.rxqs || rxq < 0 {
            Err(ZCSIError::BadRxQueue)
        } else if txq > port.txqs || txq < 0 {
            Err(ZCSIError::BadTxQueue)
        } else {
            Ok(PortQueue {
                port: port.clone(),
                port_id: port.port,
                txq: txq,
                rxq: rxq,
                stats_rx: port.stats_rx[rxq as usize].clone(),
                stats_tx: port.stats_tx[txq as usize].clone(),
                _pad0: Default::default(),
                _pad1: Default::default(),
            })
        }
    }

    /// Current port ID.
    #[inline]
    pub fn name(&self) -> i32 {
        self.port
    }

    /// Get stats for an RX/TX queue pair.
    pub fn stats(&self, queue: i32) -> (usize, usize) {
        let idx = queue as usize;
        (self.stats_rx[idx].stats.load(Ordering::Relaxed), self.stats_tx[idx].stats.load(Ordering::Relaxed))
    }

    /// Create a PMD port with a given number of RX and TXQs.
    pub fn new(port: i32,
               rxqs: i32,
               txqs: i32,
               rx_cores: &[i32],
               tx_cores: &[i32],
               nrxd: i32,
               ntxd: i32,
               loopback: bool,
               tso: bool,
               csumoffload: bool)
               -> Result<Arc<PmdPort>> {

        let loopbackv = i32_from_bool(loopback);
        let tsov = i32_from_bool(tso);
        let csumoffloadv = i32_from_bool(csumoffload);

        if ((txqs as usize) == tx_cores.len()) && ((rxqs as usize) == rx_cores.len()) {
            let ret = unsafe {
                init_pmd_port(port,
                              rxqs,
                              txqs,
                              rx_cores.as_ptr(),
                              tx_cores.as_ptr(),
                              nrxd,
                              ntxd,
                              loopbackv,
                              tsov,
                              csumoffloadv)
            };
            if ret == 0 {
                Ok(Arc::new(PmdPort {
                    connected: true,
                    port: port,
                    rxqs: rxqs,
                    txqs: txqs,
                    should_close: true,
                    stats_rx: (0..rxqs).map(|_| Arc::new(PmdStats::new())).collect(),
                    stats_tx: (0..txqs).map(|_| Arc::new(PmdStats::new())).collect(),
                    _pad: Default::default(),
                }))
            } else {
                Err(ZCSIError::FailedToInitializePort)
            }
        } else {
            Err(ZCSIError::FailedToInitializePort)
        }
    }

    pub fn new_with_one_queue(port: i32,
                              rxcore: i32,
                              txcore: i32,
                              nrxd: i32,
                              ntxd: i32,
                              loopback: bool,
                              tso: bool,
                              csumoffload: bool)
                              -> Result<Arc<PmdPort>> {
        let rx_cores = vec![rxcore];
        let tx_cores = vec![txcore];
        PmdPort::new(port,
                     1,
                     1,
                     &rx_cores[..],
                     &tx_cores[..],
                     nrxd,
                     ntxd,
                     loopback,
                     tso,
                     csumoffload)
    }

    pub fn new_loopback_port(port: i32, core: i32) -> Result<Arc<PmdPort>> {
        PmdPort::new_with_one_queue(port, core, core, NUM_RXD, NUM_TXD, true, false, false)
    }

    pub fn new_simple_port(port: i32, core: i32) -> Result<Arc<PmdPort>> {
        PmdPort::new_with_one_queue(port, core, core, NUM_RXD, NUM_TXD, false, false, false)
    }

    pub fn new_mq_port(port: i32, rxqs: i32, txqs: i32, rx_cores: &[i32], tx_cores: &[i32]) -> Result<Arc<PmdPort>> {
        PmdPort::new(port,
                     rxqs,
                     txqs,
                     &rx_cores[..],
                     &tx_cores[..],
                     NUM_RXD,
                     NUM_TXD,
                     false,
                     false,
                     false)
    }

    /// Create a new port that can talk to BESS.
    fn new_bess_port(name: &str, core: i32) -> Result<Arc<PmdPort>> {
        // This call returns the port number
        let port = unsafe {
            // This bit should not be required, but is an unfortunate problem with DPDK today.
            init_bess_eth_ring(name.as_ptr(), core)
        };
        // FIXME: Can we really not close?
        if port >= 0 {
            Ok(Arc::new(PmdPort {
                connected: true,
                port: port,
                rxqs: 1,
                txqs: 1,
                should_close: false,
                stats_rx: vec![Arc::new(PmdStats::new())],
                stats_tx: vec![Arc::new(PmdStats::new())],
                _pad: Default::default(),
            }))
        } else {
            Err(ZCSIError::FailedToInitializePort)
        }
    }

    fn new_ovs_port(name: &str, core: i32) -> Result<Arc<PmdPort>> {
        match name.parse() {
            Ok(iface) => {
                // This call returns the port number
                let port = unsafe { init_ovs_eth_ring(iface, core) };
                if port >= 0 {
                    Ok(Arc::new(PmdPort {
                        connected: true,
                        port: port,
                        rxqs: 1,
                        txqs: 1,
                        should_close: false,
                        stats_rx: vec![Arc::new(PmdStats::new())],
                        stats_tx: vec![Arc::new(PmdStats::new())],
                        _pad: Default::default(),
                    }))
                } else {
                    Err(ZCSIError::FailedToInitializePort)
                }
            }
            _ => Err(ZCSIError::BadVdev),
        }
    }

    pub fn new_vdev(name: &str, core: i32) -> Result<Arc<PmdPort>> {
        let parts: Vec<_> = name.split(':').collect();
        if parts.len() == 2 {
            match parts[0] {
                "bess" => PmdPort::new_bess_port(parts[1], core),
                "ovs" => PmdPort::new_ovs_port(parts[1], core), 
                _ => Err(ZCSIError::BadVdev),
            }
        } else {
            Err(ZCSIError::BadVdev)
        }
    }

    pub fn null_port() -> Result<Arc<PmdPort>> {
        Ok(Arc::new(PmdPort {
            connected: false,
            port: 0,
            rxqs: 0,
            txqs: 0,
            should_close: false,
            stats_rx: vec![Arc::new(PmdStats::new())],
            stats_tx: vec![Arc::new(PmdStats::new())],
            _pad: Default::default(),
        }))
    }

    #[inline]
    pub fn mac_address(&self) -> MacAddress {
        let mut address = MacAddress { addr: [0; 6] };
        unsafe {
            rte_eth_macaddr_get(self.port, &mut address as *mut MacAddress);
            address
        }
    }
}
