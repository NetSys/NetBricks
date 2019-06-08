#![allow(clippy::too_many_arguments)]

use super::super::{PacketRx, PacketTx};
use super::PortStats;
use allocators::*;
use common::*;
use config::{PortConfiguration, NUM_RXD, NUM_TXD};
use failure::Fail;
use native::mbuf::MBuf;
use native::zcsi;
use packets::MacAddr;
use regex::Regex;
use std::cmp::min;
use std::ffi::CString;
use std::fmt;
use std::sync::atomic::Ordering;
use std::sync::Arc;

/// Errors related to DPDK based PMD port transmission/receive queues.
#[derive(Debug, Fail)]
pub enum PmdPortError {
    #[fail(display = "Failed to initialize port: {}", _0)]
    FailedToInitializePort(i32),

    #[fail(display = "Cannot find device: {}", _0)]
    BadDevice(String),

    #[fail(display = "Bad virtual device specification: {}", _0)]
    BadVirtualDevice(String),

    #[fail(display = "Bad TX queue {} for port {}", _0, _1)]
    BadTxQueue(i32, i32),

    #[fail(display = "Bad RX queue {} for port {}", _0, _1)]
    BadRxQueue(i32, i32),
}

/// A DPDK based PMD port. Send and receive should not be called directly on
/// this structure but on the port queue structure instead.
pub struct PmdPort {
    connected: bool,
    should_close: bool,
    port: i32,
    rxqs: i32,
    txqs: i32,
    stats_rx: Vec<Arc<CacheAligned<PortStats>>>,
    stats_tx: Vec<Arc<CacheAligned<PortStats>>>,
}

/// A port queue represents a single queue for a physical port, and should be used to send and receive data.
#[derive(Clone)]
pub struct PortQueue {
    // The Arc cost here should not affect anything, since we are really not doing anything to make it go in and out of
    // scope.
    pub port: Arc<PmdPort>,
    stats_rx: Arc<CacheAligned<PortStats>>,
    stats_tx: Arc<CacheAligned<PortStats>>,
    port_id: i32,
    txq: i32,
    rxq: i32,
}

impl Drop for PmdPort {
    fn drop(&mut self) {
        if self.connected && self.should_close {
            unsafe {
                zcsi::free_pmd_port(self.port);
            }
        }
    }
}

/// Print information about PortQueue
impl fmt::Display for PortQueue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "port: {} ({}) rxq: {} txq: {}",
            self.port.mac_address(),
            self.port_id,
            self.rxq,
            self.txq
        )
    }
}

/// Represents a single RX/TX queue pair for a port. This is what is needed to send or receive traffic.
impl PortQueue {
    #[inline]
    fn send_queue(&self, queue: i32, pkts: *mut *mut MBuf, to_send: i32) -> Result<u32> {
        unsafe {
            let sent = zcsi::send_pkts(self.port_id, queue, pkts, to_send);
            let update = self.stats_tx.stats.load(Ordering::Relaxed) + sent as usize;
            self.stats_tx.stats.store(update, Ordering::Relaxed);
            Ok(sent as u32)
        }
    }

    #[inline]
    fn recv_queue(&self, queue: i32, pkts: *mut *mut MBuf, to_recv: i32) -> Result<u32> {
        unsafe {
            let recv = zcsi::recv_pkts(self.port_id, queue, pkts, to_recv);
            let update = self.stats_rx.stats.load(Ordering::Relaxed) + recv as usize;
            self.stats_rx.stats.store(update, Ordering::Relaxed);
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

impl PacketTx for PortQueue {
    /// Send a batch of packets out this PortQueue. Note this method is internal
    /// to NetBricks (should not be directly called).
    #[inline]
    fn send(&self, pkts: &mut [*mut MBuf]) -> Result<u32> {
        let txq = self.txq;
        let len = pkts.len() as i32;
        self.send_queue(txq, pkts.as_mut_ptr(), len)
    }
}

impl PacketRx for PortQueue {
    /// Receive a batch of packets out this PortQueue. Note this method is
    /// internal to NetBricks (should not be directly called).
    #[inline]
    fn recv(&self, pkts: &mut [*mut MBuf]) -> Result<u32> {
        let rxq = self.rxq;
        let len = pkts.len() as i32;
        self.recv_queue(rxq, pkts.as_mut_ptr(), len)
    }
}

// Utility function to go from Rust bools to C ints. Allowing match bools since this looks nicer to me.
#[cfg_attr(feature = "dev", allow(match_bool))]
#[inline]
fn i32_from_bool(x: bool) -> i32 {
    if x {
        1
    } else {
        0
    }
}

impl PmdPort {
    /// Determine the number of ports in a system.
    pub fn num_pmd_ports() -> i32 {
        unsafe { zcsi::num_pmd_ports() }
    }

    /// Find a port ID given a PCI-E string.
    pub fn find_port_id(pcie: &str) -> i32 {
        let pcie_cstr = CString::new(pcie).unwrap();
        unsafe { zcsi::find_port_with_pci_address(pcie_cstr.as_ptr()) }
    }

    /// Number of configured RXQs.
    pub fn rxqs(&self) -> i32 {
        self.rxqs
    }

    /// Number of configured TXQs.
    pub fn txqs(&self) -> i32 {
        self.txqs
    }

    pub fn new_queue_pair(
        port: &Arc<PmdPort>,
        rxq: i32,
        txq: i32,
    ) -> Result<CacheAligned<PortQueue>> {
        if rxq > port.rxqs || rxq < 0 {
            Err(PmdPortError::BadRxQueue(port.port, rxq).into())
        } else if txq > port.txqs || txq < 0 {
            Err(PmdPortError::BadTxQueue(port.port, txq).into())
        } else {
            Ok(CacheAligned::allocate(PortQueue {
                port: port.clone(),
                port_id: port.port,
                txq,
                rxq,
                stats_rx: port.stats_rx[rxq as usize].clone(),
                stats_tx: port.stats_tx[txq as usize].clone(),
            }))
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
        (
            self.stats_rx[idx].stats.load(Ordering::Relaxed),
            self.stats_tx[idx].stats.load(Ordering::Relaxed),
        )
    }

    /// Create a PMD port with a given number of RX and TXQs.
    fn init_dpdk_port(
        port: i32,
        rxqs: i32,
        txqs: i32,
        rx_cores: &[i32],
        tx_cores: &[i32],
        nrxd: i32,
        ntxd: i32,
        loopback: bool,
        tso: bool,
        csumoffload: bool,
    ) -> Result<Arc<PmdPort>> {
        let loopbackv = i32_from_bool(loopback);
        let tsov = i32_from_bool(tso);
        let csumoffloadv = i32_from_bool(csumoffload);
        let max_txqs = unsafe { zcsi::max_txqs(port) };
        let max_rxqs = unsafe { zcsi::max_rxqs(port) };
        let actual_rxqs = min(max_rxqs, rxqs);
        let actual_txqs = min(max_txqs, txqs);

        if ((actual_txqs as usize) <= tx_cores.len()) && ((actual_rxqs as usize) <= rx_cores.len())
        {
            let ret = unsafe {
                zcsi::init_pmd_port(
                    port,
                    actual_rxqs,
                    actual_txqs,
                    rx_cores.as_ptr(),
                    tx_cores.as_ptr(),
                    nrxd,
                    ntxd,
                    loopbackv,
                    tsov,
                    csumoffloadv,
                )
            };
            if ret == 0 {
                Ok(Arc::new(PmdPort {
                    connected: true,
                    port,
                    rxqs: actual_rxqs,
                    txqs: actual_txqs,
                    should_close: true,
                    stats_rx: (0..rxqs).map(|_| Arc::new(PortStats::new())).collect(),
                    stats_tx: (0..txqs).map(|_| Arc::new(PortStats::new())).collect(),
                }))
            } else {
                Err(PmdPortError::FailedToInitializePort(port).into())
            }
        } else {
            Err(PmdPortError::FailedToInitializePort(port).into())
        }
    }

    /// Create a new port that can talk to BESS.
    fn new_bess_port(name: &str, core: i32) -> Result<Arc<PmdPort>> {
        let ifname = CString::new(name).unwrap();
        // This call returns the port number
        let port = unsafe {
            // This bit should not be required, but is an unfortunate problem with DPDK today.
            zcsi::init_bess_eth_ring(ifname.as_ptr(), core)
        };
        // FIXME: Can we really not close?
        if port >= 0 {
            Ok(Arc::new(PmdPort {
                connected: true,
                port,
                rxqs: 1,
                txqs: 1,
                should_close: false,
                stats_rx: vec![Arc::new(PortStats::new())],
                stats_tx: vec![Arc::new(PortStats::new())],
            }))
        } else {
            Err(PmdPortError::FailedToInitializePort(port).into())
        }
    }

    fn new_ovs_port(name: &str, core: i32) -> Result<Arc<PmdPort>> {
        match name.parse() {
            Ok(iface) => {
                // This call returns the port number
                let port = unsafe { zcsi::init_ovs_eth_ring(iface, core) };
                if port >= 0 {
                    Ok(Arc::new(PmdPort {
                        connected: true,
                        port,
                        rxqs: 1,
                        txqs: 1,
                        should_close: false,
                        stats_rx: vec![Arc::new(PortStats::new())],
                        stats_tx: vec![Arc::new(PortStats::new())],
                    }))
                } else {
                    Err(PmdPortError::FailedToInitializePort(port).into())
                }
            }
            _ => Err(PmdPortError::BadVirtualDevice(String::from(name)).into()),
        }
    }

    fn new_dpdk_port(
        spec: &str,
        rxqs: i32,
        txqs: i32,
        rx_cores: &[i32],
        tx_cores: &[i32],
        nrxd: i32,
        ntxd: i32,
        loopback: bool,
        tso: bool,
        csumoffload: bool,
    ) -> Result<Arc<PmdPort>> {
        let cannonical_spec = PmdPort::cannonicalize_pci(spec);
        let port = unsafe { zcsi::attach_pmd_device((cannonical_spec[..]).as_ptr()) };
        if port >= 0 {
            info!("Going to try and use port {}", port);
            PmdPort::init_dpdk_port(
                port,
                rxqs,
                txqs,
                rx_cores,
                tx_cores,
                nrxd,
                ntxd,
                loopback,
                tso,
                csumoffload,
            )
            .map_err(|_| PmdPortError::BadDevice(String::from(spec)).into())
        } else {
            Err(PmdPortError::BadDevice(String::from(spec)).into())
        }
    }

    fn null_port() -> Result<Arc<PmdPort>> {
        Ok(Arc::new(PmdPort {
            connected: false,
            port: 0,
            rxqs: 0,
            txqs: 0,
            should_close: false,
            stats_rx: vec![Arc::new(PortStats::new())],
            stats_tx: vec![Arc::new(PortStats::new())],
        }))
    }

    /// Create a new port from a `PortConfiguration`.
    pub fn new_port_from_configuration(port_config: &PortConfiguration) -> Result<Arc<PmdPort>> {
        PmdPort::new_port_with_queues_descriptors_offloads(
            &port_config.name[..],
            port_config.rx_queues.len() as i32,
            port_config.tx_queues.len() as i32,
            &port_config.rx_queues[..],
            &port_config.tx_queues[..],
            port_config.rxd,
            port_config.txd,
            port_config.loopback,
            port_config.tso,
            port_config.csum,
        )
    }

    /// Create a new port.
    ///
    /// Description
    /// -   `name`: The name for a port. NetBricks currently supports Bess
    ///             native vports, OVS shared memory ports and
    ///     `dpdk` PMDs. DPDK PMDs can be used to input pcap (e.g., `dpdk:eth_pcap0,rx_pcap=<pcap_name>`), etc.
    /// -   `rxqs`, `txqs`: Number of RX and TX queues.
    /// -   `tx_cores`, `rx_cores`: Core affinity of where the queues will be used.
    /// -   `nrxd`, `ntxd`: RX and TX descriptors.
    pub fn new_port_with_queues_descriptors_offloads(
        name: &str,
        rxqs: i32,
        txqs: i32,
        rx_cores: &[i32],
        tx_cores: &[i32],
        nrxd: i32,
        ntxd: i32,
        loopback: bool,
        tso: bool,
        csumoffload: bool,
    ) -> Result<Arc<PmdPort>> {
        let parts: Vec<_> = name.splitn(2, ':').collect();
        match parts[0] {
            "bess" => PmdPort::new_bess_port(parts[1], rx_cores[0]),
            "ovs" => PmdPort::new_ovs_port(parts[1], rx_cores[0]),
            "dpdk" => PmdPort::new_dpdk_port(
                parts[1],
                rxqs,
                txqs,
                rx_cores,
                tx_cores,
                nrxd,
                ntxd,
                loopback,
                tso,
                csumoffload,
            ),
            "null" => PmdPort::null_port(),
            _ => PmdPort::new_dpdk_port(
                name,
                rxqs,
                txqs,
                rx_cores,
                tx_cores,
                nrxd,
                ntxd,
                loopback,
                tso,
                csumoffload,
            ),
        }
    }

    pub fn new_with_queues(
        name: &str,
        rxqs: i32,
        txqs: i32,
        rx_cores: &[i32],
        tx_cores: &[i32],
    ) -> Result<Arc<PmdPort>> {
        PmdPort::new_port_with_queues_descriptors_offloads(
            name, rxqs, txqs, rx_cores, tx_cores, NUM_RXD, NUM_TXD, false, false, false,
        )
    }

    pub fn new_with_cores(name: &str, rx_core: i32, tx_core: i32) -> Result<Arc<PmdPort>> {
        let rx_vec = vec![rx_core];
        let tx_vec = vec![tx_core];
        PmdPort::new_with_queues(name, 1, 1, &rx_vec[..], &tx_vec[..])
    }

    pub fn new(name: &str, core: i32) -> Result<Arc<PmdPort>> {
        PmdPort::new_with_cores(name, core, core)
    }

    fn cannonicalize_pci(pci: &str) -> CString {
        lazy_static! {
            static ref PCI_RE: Regex = Regex::new(r"^\d{2}:\d{2}\.\d$").unwrap();
        }
        if PCI_RE.is_match(pci) {
            CString::new(format!("0000:{}", pci)).unwrap()
        } else {
            CString::new(String::from(pci)).unwrap()
        }
    }

    #[inline]
    pub fn mac_address(&self) -> MacAddr {
        let mut address = MacAddr::new(0, 0, 0, 0, 0, 0);
        unsafe {
            zcsi::rte_eth_macaddr_get(self.port, &mut address as *mut MacAddr);
            address
        }
    }
}
