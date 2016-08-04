use common::*;
use interface::*;
use io::*;
use queues::*;
use std::result;
use super::act::Act;
use super::Batch;
use super::iterator::{BatchIterator, PacketDescriptor};
use headers::NullHeader;

/// Base packet batch structure, this represents an array of mbufs and is the primary interface for sending and
/// receiving packets from DPDK, allocations, etc. As a result many of the actions implemented in other types of batches
/// ultimately call into this structure.
pub struct PacketBatch {
    array: Vec<*mut MBuf>,
    cnt: i32,
    scratch: Vec<*mut MBuf>,
}

// *mut MBuf is not send by default.
unsafe impl Send for PacketBatch {}

impl PacketBatch {
    /// Create a new PacketBatch capable of holding up to `cnt` packets.
    pub fn new(cnt: i32) -> PacketBatch {
        PacketBatch {
            array: Vec::<*mut MBuf>::with_capacity(cnt as usize),
            cnt: cnt,
            scratch: Vec::<*mut MBuf>::with_capacity(cnt as usize),
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
        self.array.len()
    }

    /// The maximum number of packets that can be allocated in this batch, just returns `self.cnt`.
    #[inline]
    pub fn max_size(&self) -> i32 {
        self.cnt
    }

    /// Receive packets from a PMD port queue.
    #[inline]
    pub fn recv(&mut self, port: &mut PortQueue) -> Result<u32> {
        unsafe {
            match self.deallocate_batch() {
                Err(err) => Err(err),
                Ok(_) => self.recv_internal(port),
            }
        }
    }

    // Assumes we have already deallocated batch.
    #[inline]
    unsafe fn recv_internal(&mut self, port: &mut PortQueue) -> Result<u32> {
        match port.recv(self.packet_ptr(), self.max_size() as i32) {
            e @ Err(_) => e,
            Ok(recv) => {
                self.add_to_batch(recv as usize);
                Ok(recv)
            }
        }
    }

    #[inline]
    pub fn recv_queue<T: ReceivableQueue>(&mut self, queue: &T) -> Result<u32> {
        match self.deallocate_batch() {
            Err(err) => Err(err),
            Ok(_) => self.recv_queue_internal(queue),
        }
    }

    #[inline]
    fn recv_queue_internal<T: ReceivableQueue>(&mut self, queue: &T) -> Result<u32> {
        unsafe {
            self.array.set_len(self.cnt as usize); // Try getting as many mbufs as possible.
            let received = queue.receive_batch(&mut self.array);
            self.add_to_batch(received); // Record how many we received
            Ok(received as u32)
        }
    }

    /// This drops packet buffers and keeps things ordered. We expect that idxes is an ordered vector of indices, no
    /// guarantees are made when this is not the case.
    #[inline]
    fn drop_packets_stable(&mut self, idxes: &[usize]) -> Option<usize> {
        // Short circuit when we don't have to do this work.
        if idxes.is_empty() {
            return Some(0);
        }
        unsafe {
            let mut idx_orig = 0;
            let mut idx_new = 0;
            let mut remove_idx = 0;
            let end = self.array.len();

            // First go through the list of indexes to be filtered and get rid of them.
            while idx_orig < end && (remove_idx < idxes.len()) {
                let test_idx: usize = idxes[remove_idx];
                assert!(idx_orig <= test_idx);
                if idx_orig == test_idx {
                    self.scratch.push(self.array[idx_orig]);
                    remove_idx += 1;
                } else {
                    self.array[idx_new] = self.array[idx_orig];
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
                self.array.set_len(idx_new);
                if self.scratch.is_empty() {
                    Some(0)
                } else {
                    // Now free the dropped packets
                    let len = self.scratch.len();
                    // No need to offset here since self.scratch is tight.
                    let array_ptr = self.scratch.as_mut_ptr();
                    let ret = mbuf_free_bulk(array_ptr, (len as i32));
                    self.scratch.clear();
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
        self.array.as_mut_ptr()
    }

    #[inline]
    unsafe fn consume_batch_partial(&mut self, consumed: usize) {
        let len = self.array.len();
        for (new_idx, idx) in (consumed..len).enumerate() {
            self.array[new_idx] = self.array[idx];
        }

        self.array.set_len(len - consumed);
    }

    #[inline]
    unsafe fn consume_batch(&mut self) {
        self.array.set_len(0)
    }

    #[inline]
    unsafe fn add_to_batch(&mut self, added: usize) {
        self.array.set_len(added);
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
            if self.array.is_empty() {
                Ok(())
            } else {
                let parray = self.packet_ptr();
                let ret = mbuf_free_bulk(parray, (self.array.len() as i32));
                // If free fails, I am not sure we can do much to recover this batch.
                self.array.set_len(0);
                if ret == 0 {
                    Ok(())
                } else {
                    Err(())
                }
            }
        }
    }
}

// A packet batch is also a batch (just a special kind)
impl BatchIterator for PacketBatch {
    /// The starting offset for packets in the current batch.
    type Header = NullHeader;
    unsafe fn next_payload(&mut self, idx: usize) -> Option<PacketDescriptor<NullHeader>> {
        if idx < self.array.len() {
            Some(PacketDescriptor { packet: packet_from_mbuf_no_free::<NullHeader>(self.array[idx], 0) })
        } else {
            None
        }
    }
    #[inline]
    fn start(&mut self) -> usize {
        0
    }
}

/// Internal interface for packets.
impl Act for PacketBatch {
    #[inline]
    fn act(&mut self) {}

    #[inline]
    fn done(&mut self) {}

    #[inline]
    fn send_q(&mut self, port: &mut PortQueue) -> Result<u32> {
        let mut total_sent = 0;
        // FIXME: Make it optionally possible to wait for all packets to be sent.
        while self.available() > 0 {
            unsafe {
                match port.send(self.packet_ptr(), self.available() as i32)
                    .and_then(|sent| {
                        self.consume_batch_partial(sent as usize);
                        Ok(sent)
                    }) {
                    Ok(sent) => total_sent += sent,
                    e => return e,
                }
            }
            break;
        }
        Ok(total_sent)
    }

    #[inline]
    fn capacity(&self) -> i32 {
        self.max_size()
    }

    #[inline]
    fn drop_packets(&mut self, idxes: &[usize]) -> Option<usize> {
        self.drop_packets_stable(idxes)
    }

    #[inline]
    fn clear_packets(&mut self) {
        unsafe { self.consume_batch() }
    }

    #[inline]
    fn get_packet_batch(&mut self) -> &mut PacketBatch {
        self
    }
}

impl Batch for PacketBatch {}

impl Drop for PacketBatch {
    fn drop(&mut self) {
        let _ = self.free_packet_batch();
    }
}

// Some low level functions that need access to private members.
#[link(name = "zcsi")]
extern "C" {
    fn mbuf_alloc_bulk(array: *mut *mut MBuf, len: u16, cnt: i32) -> i32;
    fn mbuf_free_bulk(array: *mut *mut MBuf, cnt: i32) -> i32;
}
