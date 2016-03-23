use super::packet_batch::cast_from_u8;
use std::marker::PhantomData;
use super::super::interface::EndOffset;
/// An interface implemented by all batches for iterating through the set of packets in a batch.
/// This is private to the framework and not exposed.
///
/// # Safety
/// These methods are marked unsafe since they return pointers to packet mbufs. As long as packet mbufs are treated
/// correctly (i.e., assumed freed after send, freed correctly, allocated correctly, etc.) this should be safe.
/// Furthermore, dropping a packet might result in unexpected behavior (e.g., packets being skipped) but will not result
/// in crashes. Generally, do not drop or move packets during iteration, it is safer to collect the list/set of
/// packets to be modified and apply this modification later. Everything about iterator invalidation is likely to change
/// later.
pub trait BatchIterator {
    /// Returns the starting index for the packet batch. This allows for cases where the head of the batch is not at
    /// index 0.
    fn start(&mut self) -> usize;

    /// If packets are available, returns the address of the header at index `idx` in the current batch, and the index
    /// for the next packet to be processed. If packets are not available returns None. N.B., header address depends on
    /// the number of parse nodes and composition nodes seen so far.
    unsafe fn next_address(&mut self, idx: usize) -> Option<(*mut u8, usize)>;

    /// If packets are available, returns the address of the payload at index `idx` in the current batch, and the index
    /// for the next packet to be processed. If packets are not available returns None. N.B., header address depends on
    /// the number of parse nodes and composition nodes seen so far.
    unsafe fn next_payload(&mut self, idx: usize) -> Option<(*mut u8, usize)>;

    /// If packets are available, returns the address of the mbuf data_address. This is mostly to allow chained NFs to
    /// begin accessing data from the beginning. Other semantics are identical to `next_address` above.
    unsafe fn next_base_address(&mut self, idx: usize) -> Option<(*mut u8, usize)>;

    /// If packets are available, returns the address of the mbuf data_address. This is mostly to allow chained NFs to
    /// begin accessing data from the beginning. Other semantics are identical to `next_address` above.
    unsafe fn next_base_payload(&mut self, idx: usize) -> Option<(*mut u8, usize)>;
}

/// Iterate over packets in a batch. This iterator merely returns the header from the packet, and expects that
/// applications are agnostic to the index for a packet. N.B., this should be used with a for-loop.
pub struct PacketBatchIterator<'a, T>
    where T: 'a + EndOffset
{
    batch: &'a mut BatchIterator,
    idx: usize,
    phantom: PhantomData<T>,
}

impl<'a, T> PacketBatchIterator<'a, T>
    where T: 'a + EndOffset
{
    #[inline]
    pub fn new(batch: &mut BatchIterator) -> PacketBatchIterator<T> {
        let start = batch.start();
        PacketBatchIterator {
            batch: batch,
            idx: start,
            phantom: PhantomData,
        }
    }
}

impl<'a, T> Iterator for PacketBatchIterator<'a, T>
    where T: 'a + EndOffset
{
    type Item = &'a mut T;

    #[inline]
    fn next(&mut self) -> Option<&'a mut T> {
        let item = unsafe { self.batch.next_address(self.idx) };
        match item {
            Some((addr, idx)) => {
                let packet = cast_from_u8::<T>(addr);
                self.idx = idx;
                Some(packet)
            }
            None => None,
        }
    }
}

/// Enumerate packets in a batch, i.e., return both index and packet. Note, the index is meaningless outside of this
/// particular batch, in particular the index does not reveal how many packets are in a batch (batch might not start
/// from the beginning), is not guaranteed to arrive in order (one possible way of implementing grouping), might not be
/// sequential (lazy filtering), etc. Please do not use the index for anything other than as a handle to packets.
pub struct PacketBatchEnumerator<'a, T>
    where T: 'a + EndOffset
{
    batch: &'a mut BatchIterator,
    idx: usize,
    phantom: PhantomData<T>,
}

impl<'a, T> PacketBatchEnumerator<'a, T>
    where T: 'a + EndOffset
{
    #[inline]
    pub fn new(batch: &mut BatchIterator) -> PacketBatchEnumerator<T> {
        let start = batch.start();
        PacketBatchEnumerator {
            batch: batch,
            idx: start,
            phantom: PhantomData,
        }
    }
}

impl<'a, T> Iterator for PacketBatchEnumerator<'a, T>
    where T: 'a + EndOffset
{
    type Item = (usize, &'a mut T);

    #[inline]
    fn next(&mut self) -> Option<(usize, &'a mut T)> {
        let original_idx = self.idx;
        let item = unsafe { self.batch.next_address(original_idx) };
        match item {
            Some((addr, next_idx)) => {
                let packet = cast_from_u8::<T>(addr);
                self.idx = next_idx;
                Some((original_idx, packet))
            }
            None => None,
        }
    }
}
