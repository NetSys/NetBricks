use std::result;
#[repr(C)]
pub struct MBuf {
    buf_addr: *mut u8,
    phys_addr: usize,
    buf_len: u16,
    data_off: u16,
    refcnt: u16,
    nb_segs: u8,
    port: u8,
    ol_flags: u8,
    packet_type: u32,
    pkt_len: u32,
    data_len: u16,
    vlan_tci: u16,
    hash: u64,
    seqn: u32,
    vlan_tci_outer: u32,
    userdata: u64,
    pool: u64,
    next: *mut MBuf,
    tx_offload: u64,
    priv_size: u16,
    timesync:u16
}

impl MBuf {
    pub fn data_address(&self) -> *mut u8 {
        unsafe {
            self.buf_addr.offset(self.data_off as isize)
        }
    }
}

pub trait FromMBuf {
    fn mut_transform(pkt: &mut MBuf) -> &mut Self;
    fn const_transform(pkt: &MBuf) -> &Self;
}

#[derive(Debug)]
pub enum ZCSIError {
    FailedAllocation,
    FailedDeallocation,
    FailedToInitializePort,
    BadQueue,
}

pub type Result<T> = result::Result<T, ZCSIError>;

