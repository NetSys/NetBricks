#[repr(C)]
pub struct MBuf {
    buf_addr: *mut u8,
    phys_addr: usize,
    buf_len: u16,
    data_off: u16,
    refcnt: u16,
    nb_segs: u8,
    port: u8,
    ol_flags: u64,
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
    timesync: u16,
}

// FIXME: Remove this once we start using these functions correctly
#[allow(dead_code)]
impl MBuf {
    #[inline]
    pub fn read_metadata_slot(mbuf: *mut MBuf, slot: usize) -> usize {
        unsafe {
            let ptr = (mbuf.offset(1) as *mut usize).offset(slot as isize);
            *ptr
        }
    }

    #[inline]
    pub fn write_metadata_slot(mbuf: *mut MBuf, slot: usize, value: usize) {
        unsafe {
            let ptr = (mbuf.offset(1) as *mut usize).offset(slot as isize);
            *ptr = value;
        }
    }

    #[inline]
    pub unsafe fn metadata_as<T: Sized>(mbuf: *const MBuf, slot: usize) -> *const T {
        (mbuf.offset(1) as *const usize).offset(slot as isize) as *const T
    }

    #[inline]
    pub unsafe fn mut_metadata_as<T: Sized>(mbuf: *mut MBuf, slot: usize) -> *mut T {
        (mbuf.offset(1) as *mut usize).offset(slot as isize) as *mut T
    }

    #[inline]
    pub fn data_address(&self, offset: usize) -> *mut u8 {
        unsafe {
            self.buf_addr.offset(
                self.data_off as isize + offset as isize,
            )
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

    #[inline]
    fn pkt_headroom(&self) -> usize {
        self.data_off as usize
    }

    #[inline]
    fn pkt_tailroom(&self) -> usize {
        self.buf_len() - self.data_off as usize - self.data_len()
    }

    /// Add data to the beginning of the packet. This might fail (i.e., return 0) when no more headroom is left.
    #[inline]
    pub fn add_data_beginning(&mut self, len: usize) -> usize {
        // If only we could add a likely here.
        if len > self.pkt_headroom() {
            0
        } else {
            self.data_off -= len as u16;
            self.data_len += len as u16;
            self.pkt_len += len as u32;
            len
        }
    }

    /// Add data to the end of a packet buffer. This might fail (i.e., return 0) when no more tailroom is left. We do
    /// not currently deal with packet with multiple segments.
    #[inline]
    pub fn add_data_end(&mut self, len: usize) -> usize {
        if len > self.pkt_tailroom() {
            0
        } else {
            self.data_len += len as u16;
            self.pkt_len += len as u32;
            len
        }
    }

    #[inline]
    pub fn remove_data_beginning(&mut self, len: usize) -> usize {
        if len > self.data_len() {
            0
        } else {
            self.data_off += len as u16;
            self.data_len -= len as u16;
            self.pkt_len -= len as u32;
            len
        }
    }

    #[inline]
    pub fn remove_data_end(&mut self, len: usize) -> usize {
        if len > self.data_len() {
            0
        } else {
            self.data_len -= len as u16;
            self.pkt_len -= len as u32;
            len
        }
    }

    #[inline]
    pub fn refcnt(&self) -> u16 {
        self.refcnt
    }

    #[inline]
    pub fn reference(&mut self) {
        self.refcnt += 1;
    }
}
