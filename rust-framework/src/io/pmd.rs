extern crate libc;
use super::mbuf::MBuf;
use super::mbuf::Result;
use super::mbuf::ZCSIError;
use super::packet_batch::PacketBatch;
use super::packet_batch::packet_ptr;
use super::packet_batch::consumed_batch;
use super::packet_batch::add_to_batch;
#[link(name = "zcsi")]
extern {
    fn init_pmd_port(port: i32, rxqs: i32, txqs: i32, rxcores: *const i32,
                     txcores: *const i32, nrxd: i32, ntxd: i32, loopback: i32,
                     tso: i32, csumoffload: i32) -> i32;
    fn free_pmd_port(port: i32) -> i32;
    fn recv_pkts(port: i32, qid: i32, pkts: *mut *mut MBuf, len: i32) -> i32;
    fn send_pkts(port: i32, qid: i32, pkts: *mut *mut MBuf, len: i32) -> i32;
}

pub struct PmdPort {
    connected: bool,
    port: i32,
    rxqs: i32,
    txqs: i32,
}

impl Drop for PmdPort {
    fn drop(&mut self) {
        if self.connected {
            unsafe {
                free_pmd_port(self.port);
            }
            self.connected = false;
        }
    }
}

const NUM_RXD: i32 = 128;
const NUM_TXD: i32 = 512;

impl PmdPort {
    pub fn new(port: i32, rxqs: i32, txqs: i32, rxcores: &Vec<i32>,
           txcores: &Vec<i32>, nrxd: i32, ntxd: i32, loopback: bool,
           tso: bool, csumoffload: bool) -> Result<PmdPort> {
        assert_eq!(rxqs as usize, rxcores.len());
        assert_eq!(txqs as usize, txcores.len());
        let loopbackv = if loopback { 1 } else { 0 };
        let tsov = if tso { 1 } else { 0 };
        let csumoffloadv = if csumoffload { 1 } else { 0 };
        let ret = unsafe {init_pmd_port(port, rxqs, txqs, rxcores.as_ptr(), txcores.as_ptr(), nrxd,
                                        ntxd, loopbackv, tsov, csumoffloadv) };
        // TODO: Switch to checking and doing things.
        if ret == 0 {
            Ok(PmdPort {connected: true, port: port, rxqs: rxqs, txqs: txqs})
        } else {
            Err(ZCSIError::FailedToInitializePort)
        }
    }

    pub fn new_with_one_queue(port: i32, rxcore: i32, txcore: i32, nrxd: i32, ntxd: i32, loopback: bool,
                       tso: bool, csumoffload: bool) -> Result<PmdPort> {
        let rxcores = vec![rxcore];
        let txcores = vec![txcore];
        PmdPort::new(port, 1, 1, &rxcores, &txcores, nrxd, ntxd, loopback, tso, csumoffload)
    }

    pub fn new_loopback_port(port: i32, core: i32) -> Result<PmdPort> {
        PmdPort::new_with_one_queue(port, core, core, NUM_RXD, NUM_TXD, true, false, false)
    }

    pub fn new_simple_port(port: i32, core: i32) -> Result<PmdPort> {
        PmdPort::new_with_one_queue(port, core, core, NUM_RXD, NUM_TXD, false, false, false)
    } 

    #[inline]
    pub fn send(&self, pkts: &mut PacketBatch) -> u32 {
        unsafe {
            let to_send = pkts.available() as i32;
            let sent = send_pkts(self.port, 0, packet_ptr(pkts), to_send);
            consumed_batch(pkts, sent as usize);
            sent as u32
        }
    }

    #[inline]
    pub fn recv(&self, pkts: &mut PacketBatch) -> Result<u32> {
        unsafe {
            match pkts.deallocate_batch() {
                Err(err) => Err(err),
                Ok(_) => { let to_recv = pkts.max_size();
                         let recv = recv_pkts(self.port, 0, packet_ptr(pkts), to_recv);
                         add_to_batch(pkts, recv as usize);
                         Ok(recv as u32) 
                }
            }
        }
    }

    #[inline]
    pub fn send_queue(&self, queue: i32, pkts: &mut PacketBatch) -> Result<u32> {
        if self.txqs < queue {
            Err(ZCSIError::BadQueue)
        } else {
            unsafe {
                let to_send = pkts.available() as i32;
                let sent = send_pkts(self.port, 0, packet_ptr(pkts), to_send);
                consumed_batch(pkts, sent as usize);
                Ok(sent as u32)
            }
        }
    }

    #[inline]
    pub fn recv_queue(&self, queue: i32, pkts: &mut PacketBatch) -> Result<u32> {
        if self.rxqs < queue {
            Err(ZCSIError::BadQueue)
        } else {
            unsafe {
                match pkts.deallocate_batch() {
                    Err(err) => Err(err),
                    Ok(_) => { let to_recv = pkts.max_size();
                             let recv = recv_pkts(self.port, 0, packet_ptr(pkts), to_recv);
                             add_to_batch(pkts, recv as usize);
                             Ok(recv as u32) 
                    }
                }
            }
        }
    }
}
