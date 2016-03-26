use super::packet_batch::cast_from_u8;
use std::marker::PhantomData;
use super::super::interface::EndOffset;
use std::any::Any;
use std::cell::Cell;
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
    unsafe fn next_address(&mut self, idx: usize) -> Option<(*mut u8, Option<&mut Any>, usize)>;

    /// If packets are available, returns the address of the payload at index `idx` in the current batch, and the index
    /// for the next packet to be processed. If packets are not available returns None. N.B., header address depends on
    /// the number of parse nodes and composition nodes seen so far.
    unsafe fn next_payload(&mut self, idx: usize) -> Option<(*mut u8, Option<&mut Any>, usize)>;

    /// If packets are available, returns the address of the mbuf data_address. This is mostly to allow chained NFs to
    /// begin accessing data from the beginning. Other semantics are identical to `next_address` above.
    unsafe fn next_base_address(&mut self, idx: usize) -> Option<(*mut u8, Option<&mut Any>, usize)>;

    /// If packets are available, returns the address of the mbuf data_address. This is mostly to allow chained NFs to
    /// begin accessing data from the beginning. Other semantics are identical to `next_address` above.
    unsafe fn next_base_payload(&mut self, idx: usize) -> Option<(*mut u8, Option<&mut Any>, usize)>;
}

/// Iterate over packets in a batch. This iterator merely returns the header from the packet, and expects that
/// applications are agnostic to the index for a packet. N.B., this should be used with a for-loop.
pub struct PacketBatchIterator<T>
    where T: EndOffset
{
    idx: Cell<usize>,
    phantom: PhantomData<T>,
}

impl<T> PacketBatchIterator<T>
    where T: EndOffset
{
    #[inline]
    pub fn new(batch: &mut BatchIterator) -> PacketBatchIterator<T> {
        let start = batch.start();
        PacketBatchIterator {
            idx: Cell::new(start),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn next<'a>(&'a self, batch: &'a mut BatchIterator) -> Option<(&'a mut T, Option<&'a mut Any>)> {
        let item = unsafe { batch.next_address(self.idx.get()) };
        match item {
            Some((addr, ctx, idx)) => {
                let packet = cast_from_u8::<T>(addr);
                self.idx.set(idx);
                Some((packet, ctx))
            }
            None => None,
        }
    }
}

/// Enumerate packets in a batch, i.e., return both index and packet. Note, the index is meaningless outside of this
/// particular batch, in particular the index does not reveal how many packets are in a batch (batch might not start
/// from the beginning), might not be sequential (lazy filtering), etc. We however do guarantee that the iterator will
/// present monotonically increasing indices. Please do not use the index for anything other than as a handle for
/// packets.
pub struct PacketBatchEnumerator<T>
    where T: EndOffset
{
    idx: Cell<usize>,
    phantom: PhantomData<T>,
}

impl<T> PacketBatchEnumerator<T>
    where T: EndOffset
{
    #[inline]
    pub fn new(batch: &mut BatchIterator) -> PacketBatchEnumerator<T> {
        let start = batch.start();
        PacketBatchEnumerator {
            idx: Cell::new(start),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn next<'a>(&'a self, batch: &'a mut BatchIterator) -> Option<(usize, &'a mut T, Option<&'a mut Any>)> {
        let original_idx = self.idx.get();
        let item = unsafe { batch.next_address(original_idx) };
        match item {
            Some((addr, ctx, next_idx)) => {
                let packet = cast_from_u8::<T>(addr);
                self.idx.set(next_idx);
                Some((original_idx, packet, ctx))
            }
            None => None,
        }
    }
}
