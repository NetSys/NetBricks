use super::mbuf::MBuf;
use packets::ethernet::MacAddr;
use std::os::raw::c_char;
#[link(name = "zcsi")]
extern "C" {
    pub fn init_system_whitelisted(
        name: *const c_char,
        nlen: i32,
        core: i32,
        whitelist: *mut *const c_char,
        wlcount: i32,
        pool_size: u32,
        cache_size: u32,
        slots: u16,
    ) -> i32;
    pub fn init_thread(tid: i32, core: i32) -> i32;
    pub fn init_secondary(
        name: *const c_char,
        nlen: i32,
        core: i32,
        vdevs: *mut *const c_char,
        vdev_count: i32,
    ) -> i32;
    pub fn init_pmd_port(
        port: i32,
        rxqs: i32,
        txqs: i32,
        rx_cores: *const i32,
        tx_cores: *const i32,
        nrxd: i32,
        ntxd: i32,
        loopback: i32,
        tso: i32,
        csumoffload: i32,
    ) -> i32;
    pub fn free_pmd_port(port: i32) -> i32;
    pub fn recv_pkts(port: i32, qid: i32, pkts: *mut *mut MBuf, len: i32) -> i32;
    pub fn send_pkts(port: i32, qid: i32, pkts: *mut *mut MBuf, len: i32) -> i32;
    pub fn num_pmd_ports() -> i32;
    pub fn rte_eth_macaddr_get(port: i32, address: *mut MacAddr);
    pub fn init_bess_eth_ring(ifname: *const c_char, core: i32) -> i32;
    pub fn init_ovs_eth_ring(iface: i32, core: i32) -> i32;
    pub fn find_port_with_pci_address(pciaddr: *const c_char) -> i32;
    pub fn attach_pmd_device(dev: *const c_char) -> i32;
    // FIXME: Generic PMD info
    pub fn max_rxqs(port: i32) -> i32;
    pub fn max_txqs(port: i32) -> i32;
    pub fn mbuf_alloc() -> *mut MBuf;
    pub fn mbuf_free(buf: *mut MBuf);
    pub fn mbuf_alloc_bulk(array: *mut *mut MBuf, len: u16, cnt: i32) -> i32;
    pub fn mbuf_free_bulk(array: *mut *mut MBuf, cnt: i32) -> i32;
    pub fn crc_hash_native(to_hash: *const u8, size: u32, iv: u32) -> u32;
    pub fn ipv4_cksum(payload: *const u8) -> u16;
}
