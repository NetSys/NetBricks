use super::Batch;
use super::act::Act;
use super::iterator::{BatchIterator, PacketDescriptor};
use common::*;
use headers::NullHeader;
use interface::*;
use native::zcsi::*;
use std::result;

/// Base packet batch structure, this represents an array of mbufs and is the primary interface for sending and
/// receiving packets from DPDK, allocations, etc. As a result many of the actions implemented in other types of batches
/// ultimately call into this structure.
pub struct PacketBatch {
    array: Vec<*mut MBuf>,
    scratch: Vec<*mut MBuf>,
    parent_tasks: Vec<usize>,
}

// *mut MBuf is not send by default.
unsafe impl Send for PacketBatch {}

impl PacketBatch {
    /// Create a new PacketBatch capable of holding up to `cnt` packets.
    pub fn new(cnt: i32) -> PacketBatch {
        PacketBatch {
            array: Vec::<*mut MBuf>::with_capacity(cnt as usize),
            scratch: Vec::<*mut MBuf>::with_capacity(cnt as usize),
            parent_tasks: vec![],
        }
    }

    #[inline]
    pub fn add_parent_task(&mut self, task: usize) {
        self.parent_tasks.push(task);
    }

    #[inline]
    pub fn get_parent_task(&self) -> &Vec<usize> {
        &self.parent_tasks
    }

    /// Allocate as many mbufs as batch can hold. `len` here merely sets the extent of the mbuf considered when sending
    /// a packet. We always allocate mbuf's of the same size.
    #[inline]
    pub fn allocate_batch_with_size(&mut self, len: u16) -> Result<&mut Self> {
        let capacity = self.array.capacity() as i32;
        self.alloc_packet_batch(len, capacity)
            .and_then(|_| Ok(self))
    }

    /// Allocate `cnt` mbufs. `len` sets the metadata field indicating how much of the mbuf should be considred when
    /// sending the packet, all `mbufs` are of the same size.
    #[inline]
    pub fn allocate_partial_batch_with_size(&mut self, len: u16, cnt: i32) -> Result<&mut Self> {
        match self.alloc_packet_batch(len, cnt) {
            Ok(_) => Ok(self),
            Err(_) => Err(ErrorKind::FailedAllocation.into()),
        }
    }

    /// Free all mbuf's held in this batch.
    #[inline]
    pub fn deallocate_batch(&mut self) -> Result<&mut Self> {
        match self.free_packet_batch() {
            Ok(_) => Ok(self),
            Err(_) => Err(ErrorKind::FailedDeallocation.into()),
        }
    }

    /// Number of available mbufs.
    #[inline]
    pub fn available(&self) -> usize {
        self.array.len()
    }

    /// Receive packets from a PMD port queue.
    #[inline]
    pub fn recv<Rx: PacketRx>(&mut self, port: &Rx) -> Result<u32> {
        unsafe {
            match self.deallocate_batch() {
                Err(err) => Err(err),
                Ok(_) => self.recv_internal(port),
            }
        }
    }

    // Assumes we have already deallocated batch.
    #[inline]
    unsafe fn recv_internal<Rx: PacketRx>(&mut self, port: &Rx) -> Result<u32> {
        let capacity = self.array.capacity();
        self.add_to_batch(capacity);
        match port.recv(self.packet_ptr()) {
            e @ Err(_) => e,
            Ok(recv) => {
                self.add_to_batch(recv as usize);
                Ok(recv)
            }
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
    unsafe fn packet_ptr(&mut self) -> &mut [*mut MBuf] {
        &mut self.array[..]
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
    fn alloc_packet_batch(&mut self, len: u16, cnt: i32) -> Result<()> {
        unsafe {
            if self.array.capacity() < (cnt as usize) {
                Err(ErrorKind::FailedAllocation.into())
            } else {
                let ret = mbuf_alloc_bulk(self.array.as_mut_ptr(), len, cnt);
                if ret == 0 {
                    self.array.set_len(cnt as usize);
                    Ok(())
                } else {
                    Err(ErrorKind::FailedAllocation.into())
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
                let len = self.array.len() as i32;
                let ret = {
                    let parray = self.packet_ptr().as_mut_ptr();
                    mbuf_free_bulk(parray, len)
                };
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
    type Metadata = EmptyMetadata;
    unsafe fn next_payload(&mut self, idx: usize) -> Option<PacketDescriptor<NullHeader, EmptyMetadata>> {
        if idx < self.array.len() {
            Some(PacketDescriptor {
                packet: packet_from_mbuf_no_free::<NullHeader>(self.array[idx], 0),
            })
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
    fn send_q(&mut self, port: &PacketTx) -> Result<u32> {
        let mut total_sent = 0;
        // FIXME: Make it optionally possible to wait for all packets to be sent.
        while self.available() > 0 {
            unsafe {
                // let available = self.available() as i32;
                try!(port.send(self.packet_ptr()).and_then(|sent| {
                    self.consume_batch_partial(sent as usize);
                    total_sent += sent;
                    Ok(sent)
                }));
            }
            break;
        }
        Ok(total_sent)
    }

    #[inline]
    fn capacity(&self) -> i32 {
        self.array.capacity() as i32
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

    #[inline]
    fn get_task_dependencies(&self) -> Vec<usize> {
        self.get_parent_task().clone()
    }
}

impl Batch for PacketBatch {}

impl Drop for PacketBatch {
    fn drop(&mut self) {
        let _ = self.free_packet_batch();
    }
}
