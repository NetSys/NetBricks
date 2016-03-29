use std::result;
use super::act::Act;
use super::Batch;
use super::iterator::BatchIterator;
use super::super::mbuf::*;
use super::super::interface::Result;
use super::super::interface::ZCSIError;
use super::super::interface::EndOffset;
use super::super::pmd::*;
use std::any::Any;

/// Base packet batch structure, this represents an array of mbufs and is the primary interface for sending and
/// receiving packets from DPDK, allocations, etc. As a result many of the actions implemented in other types of batches
/// ultimately call into this structure.
pub struct PacketBatch {
    array: Vec<*mut MBuf>,
    cnt: i32,
    start: usize,
}

impl PacketBatch {
    /// Create a new PacketBatch capable of holding up to `cnt` packets.
    pub fn new(cnt: i32) -> PacketBatch {
        PacketBatch {
            array: Vec::<*mut MBuf>::with_capacity(cnt as usize),
            cnt: cnt,
            start: 0,
        }
    }

    /// Allocate `self.cnt` mbufs. `len` here merely sets the extent of the mbuf considered when sending a packet. We
    /// always allocate mbuf's of the same size.
    #[inline]
    pub fn allocate_batch_with_size(&mut self, len: u16) -> Result<&mut Self> {
        let cnt = self.cnt;
        match self.alloc_packet_batch(len, cnt) {
            Ok(_) => Ok(self),
            Err(_) => Err(ZCSIError::FailedAllocation),
        }
    }

    /// Allocate `cnt` mbufs. `len` sets the metadata field indicating how much of the mbuf should be considred when
    /// sending the packet, all `mbufs` are of the same size.
    #[inline]
    pub fn allocate_partial_batch_with_size(&mut self, len: u16, cnt: i32) -> Result<&mut Self> {
        match self.alloc_packet_batch(len, cnt) {
            Ok(_) => Ok(self),
            Err(_) => Err(ZCSIError::FailedAllocation),
        }
    }

    /// Free all mbuf's held in this batch.
    #[inline]
    pub fn deallocate_batch(&mut self) -> Result<&mut Self> {
        match self.free_packet_batch() {
            Ok(_) => Ok(self),
            Err(_) => Err(ZCSIError::FailedDeallocation),
        }
    }

    /// Number of available mbufs.
    #[inline]
    pub fn available(&self) -> usize {
        (self.array.len() - self.start)
    }

    /// The maximum number of packets that can be allocated in this batch, just returns `self.cnt`.
    #[inline]
    pub fn max_size(&self) -> i32 {
        self.cnt
    }

    /// Receive packets from a PMD port queue.
    #[inline]
    pub fn recv_queue(&mut self, port: &mut PmdPort, queue: i32) -> Result<u32> {
        unsafe {
            match self.deallocate_batch() {
                Err(err) => Err(err),
                Ok(_) => self.recv_internal(port, queue),
            }
        }
    }

    /// This drops a given vector of packets. This method is O(n) in number of packets, however it does not preserve
    /// ordering. Currently hidden behind a cfg flag.
    #[cfg(unordered_drop)]
    #[inline]
    fn drop_packets_unordered(&mut self, idxes_ordered: Vec<usize>) -> Option<usize> {
        let mut idxes = idxes_ordered;
        idxes.reverse();
        let mut to_free = Vec::<*mut MBuf>::with_capacity(idxes.len());
        // First remove them from this packetbatch, compacting the batch as appropriate. Note this will not change start
        // so is safe.
        for idx in idxes {
            to_free.push(self.array.swap_remove(idx));
        }

        // Great we removed everything
        if self.start == self.array.len() {
            unsafe {
                self.start = 0;
                self.array.set_len(0);
            }
        }

        if to_free.len() == 0 {
            Some(0)
        } else {
            // Now free the dropped packets
            unsafe {
                let len = to_free.len();
                // No need to offset here since to_free is tight.
                let array_ptr = to_free.as_mut_ptr();
                let ret = mbuf_free_bulk(array_ptr, (len as i32));
                if ret == 0 {
                    Some(len)
                } else {
                    None
                }
            }
        }
    }

    /// This drops packet buffers and keeps things ordered. We expect that idxes is an ordered vector of indices, no
    /// guarantees are made when this is not the case.
    #[inline]
    fn drop_packets_stable(&mut self, idxes: Vec<usize>) -> Option<usize> {
        let mut to_free = Vec::<*mut MBuf>::with_capacity(idxes.len());
        // Short circuit when we don't have to do this work.
        if idxes.is_empty() {
            return Some(0);
        }
        unsafe {
            let mut idx_orig = self.start;
            let mut idx_new = 0;
            let mut remove_idx = 0;
            let end = self.array.len();

            // First go through the list of indexes to be filtered and get rid of them.
            while idx_orig < end && (remove_idx < idxes.len()) {
                let test_idx: usize = idxes[remove_idx];
                assert!(idx_orig <= test_idx);
                if idx_orig == test_idx {
                    to_free.push(self.array[idx_orig]);
                    remove_idx += 1;
                } else {
                    self.array.swap(idx_orig, idx_new);
                    idx_new += 1;
                }
                idx_orig += 1;
            }
            // Then copy over any left over packets.
            while idx_orig < end {
                self.array[idx_new] = self.array[idx_orig];
                idx_orig += 1;
                idx_new += 1;
            }

            // We did not find an index that was passed in, warn/error out.
            if remove_idx < idxes.len() {
                None
            } else {
                self.start = 0;
                self.array.set_len(idx_new);
                if to_free.is_empty() {
                    Some(0)
                } else {
                    // Now free the dropped packets
                    let len = to_free.len();
                    // No need to offset here since to_free is tight.
                    let array_ptr = to_free.as_mut_ptr();
                    let ret = mbuf_free_bulk(array_ptr, (len as i32));
                    if ret == 0 {
                        Some(len)
                    } else {
                        None
                    }
                }
            }
        }
    }

    // Some private utility functions.
    #[inline]
    unsafe fn packet_ptr(&mut self) -> *mut *mut MBuf {
        self.array.as_mut_ptr().offset(self.start as isize)
    }

    #[inline]
    unsafe fn consumed_batch(&mut self, consumed: usize) {
        self.start += consumed;
        if self.start == self.array.len() {
            self.start = 0;
            self.array.set_len(0);
        }
    }

    #[inline]
    unsafe fn add_to_batch(&mut self, added: usize) {
        assert_eq!(self.start, 0);
        self.start = 0;
        self.array.set_len(added);
    }

    // Assumes we have already deallocated batch.
    #[inline]
    unsafe fn recv_internal(&mut self, port: &mut PmdPort, queue: i32) -> Result<u32> {
        match port.recv_queue(queue, self.packet_ptr(), self.max_size() as i32) {
            e @ Err(_) => e,
            Ok(recv) => {
                self.add_to_batch(recv as usize);
                Ok(recv)
            }
        }
    }

    #[inline]
    fn alloc_packet_batch(&mut self, len: u16, cnt: i32) -> result::Result<(), ()> {
        unsafe {
            if self.array.capacity() < (cnt as usize) {
                Err(())
            } else {
                let parray = self.array.as_mut_ptr();
                let ret = mbuf_alloc_bulk(parray, len, cnt);
                if ret == 0 {
                    self.start = 0;
                    self.array.set_len(cnt as usize);
                    Ok(())
                } else {
                    Err(())
                }
            }
        }
    }

    #[inline]
    fn free_packet_batch(&mut self) -> result::Result<(), ()> {
        unsafe {
            if self.array.len() > self.start {
                let parray = self.packet_ptr();
                let ret = mbuf_free_bulk(parray, ((self.array.len() - self.start) as i32));
                // If free fails, I am not sure we can do much to recover this batch.
                self.start = 0;
                self.array.set_len(0);
                if ret == 0 {
                    Ok(())
                } else {
                    Err(())
                }
            } else {
                Ok(())
            }
        }
    }

    /// Return the payload for a given packet and size for a given packet.
    ///
    /// # Safety
    /// `idx` must be a valid index.
    #[inline]
    unsafe fn payload(&mut self, idx: usize) -> (*mut u8, usize) {
        let val = &mut *self.array[idx];
        (val.data_address(0), val.data_len())
    }

    /// Address for the payload for a given packet.
    #[inline]
    unsafe fn address(&mut self, idx: usize) -> (*mut u8, usize) {
        let val = &mut *self.array[idx];
        (val.data_address(0), val.data_len())
    }
}

// A packet batch is also a batch (just a special kind)
impl BatchIterator for PacketBatch {
    /// The starting offset for packets in the current batch.
    #[inline]
    fn start(&mut self) -> usize {
        self.start
    }

    /// Address for the next packet.
    /// Returns packet at index `idx` and the index of the next packet after `idx`.
    #[inline]
    unsafe fn next_address(&mut self, idx: usize, _: i32) -> address_iterator_return!{} {
        if self.start <= idx && idx < self.array.len() {
            Some((self.address(idx).0, self.address(idx).1, None, idx + 1))
        } else {
            None
        }
    }

    /// Payload for the next packet.
    #[inline]
    unsafe fn next_payload(&mut self, idx: usize) -> payload_iterator_return!{} {
        if self.start <= idx && idx < self.array.len() {
            Some((self.address(idx).0,
                  self.payload(idx).0,
                  self.payload(idx).1,
                  None,
                  idx + 1))
        } else {
            None
        }
    }

    #[inline]
    unsafe fn next_base_address(&mut self, idx: usize) -> address_iterator_return!{} {
        self.next_address(idx, 0)
    }

    #[inline]
    unsafe fn next_base_payload(&mut self, idx: usize) -> payload_iterator_return!{} {
        self.next_payload(idx)
    }
}

/// Internal interface for packets.
impl Act for PacketBatch {
    #[inline]
    fn act(&mut self) {}

    #[inline]
    fn done(&mut self) {}

    #[inline]
    fn send_queue(&mut self, port: &mut PmdPort, queue: i32) -> Result<u32> {
        unsafe {
            port.send_queue(queue, self.packet_ptr(), self.available() as i32)
                .and_then(|sent| {
                    self.consumed_batch(sent as usize);
                    Ok(sent)
                })
        }
    }

    #[inline]
    fn capacity(&self) -> i32 {
        self.max_size()
    }

    #[inline]
    fn drop_packets(&mut self, idxes: Vec<usize>) -> Option<usize> {
        self.drop_packets_stable(idxes)
    }
}

impl Batch for PacketBatch {}

impl Drop for PacketBatch {
    fn drop(&mut self) {
        let _ = self.free_packet_batch();
    }
}

#[inline]
pub fn cast_from_u8<'a, T: 'a>(data: *mut u8) -> &'a mut T {
    let typecast = data as *mut T;
    unsafe { &mut *typecast }
}

// Some low level functions that need access to private members.
#[link(name = "zcsi")]
extern "C" {
    fn mbuf_alloc_bulk(array: *mut *mut MBuf, len: u16, cnt: i32) -> i32;
    fn mbuf_free_bulk(array: *mut *mut MBuf, cnt: i32) -> i32;
}
