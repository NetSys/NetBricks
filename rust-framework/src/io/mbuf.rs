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

// FIXME: Remove this once we start using these functions correctly
#[allow(dead_code)]
impl MBuf {
    #[inline]
    pub fn data_address(&self, offset: usize) -> *mut u8 {
        unsafe {
            self.buf_addr.offset(self.data_off as isize).offset(offset as isize)
        }
    }

    /// Returns the total allocated size of this mbuf segment.
    /// This is a constant.
    #[inline]
    pub fn buf_len(&self) -> usize {
        self.buf_len as usize
    }

    /// Returns the length of data in this mbuf segment.
    #[inline]
    pub fn data_len(&self) -> usize {
        self.data_len as usize
    }

    /// Returns the size of the packet (across multiple mbuf segment).
    #[inline]
    pub fn pkt_len(&self) -> usize {
        self.pkt_len as usize
    }

    /// Change the length of data in this packet.
    ///
    /// FIXME: This does not consider segments
    #[inline]
    pub fn change_data_len(&mut self, len: usize) {
        self.pkt_len = len as u32;
        self.data_len = len as u16;
    }

    /// Add data to this packet.
    pub fn append_data(&mut self, len: usize) {
        self.pkt_len = self.pkt_len + (len as u32);
        self.data_len = self.data_len + (len as u16);
    }
}
