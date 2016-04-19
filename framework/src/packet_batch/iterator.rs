use super::packet_batch::cast_from_u8;
use std::marker::PhantomData;
use headers::EndOffset;
use std::any::Any;
use std::cell::Cell;
use std::slice::*;

/// A struct containing all the packet related information passed around by iterators.
pub struct PacketDescriptor {
    pub offset: usize,
    /// Address for the header.
    pub header: *mut u8,
    /// Address for the payload (this comes after the header).
    pub payload: *mut u8,
    /// Payload size, useful for making bounded vectors into the packet.
    pub payload_size: usize,
}

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

    /// If packets are available (i.e., `idx` is not past the end of the batch), returns the descriptor for the `idx`th
    /// packet, any associated metadata (context) and the next index. Otherwise returns None. Note this should not be
    /// used directly, use one of the nice iterators below.
    unsafe fn next_payload(&mut self, idx: usize) -> Option<(PacketDescriptor, Option<&mut Any>, usize)>;

    /// Same as above, except pop off (subtract packet offset) by as `pop` parse nodes. This allows `DeparsedBatch` to
    /// be implemented.
    unsafe fn next_payload_popped(&mut self,
                                  idx: usize,
                                  pop: i32)
                                  -> Option<(PacketDescriptor, Option<&mut Any>, usize)>;

    /// Same as above, except return addresses from the start of the packet (offset 0). This allows `ResetBatch` to be
    /// implemented.
    unsafe fn next_base_payload(&mut self, idx: usize) -> Option<(PacketDescriptor, Option<&mut Any>, usize)>;
}

/// A struct containing the parsed information returned by the `PayloadEnumerator`.
pub struct ParsedDescriptor<'a, T>
    where T: 'a + EndOffset
{
    pub index: usize,
    pub header: &'a mut T,
    pub payload: &'a mut [u8],
    pub ctx: Option<&'a mut Any>,
    /// Offset (from 0) at which the current payload resides.
    pub offset: usize,
}

/// An enumerator over both the header and the payload. The payload is represented as an appropriately sized slice of
/// bytes. The expectation is therefore that the user can operate on bytes, or make appropriate adjustments as
/// necessary.
pub struct PayloadEnumerator<T>
    where T: EndOffset
{
    // Was originally using a cell here so we didn't have to borrow the iterator mutably. I think at this point, given
    // that the batch is not stored in the iterator this might be a moot point, but it does allow the iterator to be
    // entirely immutable for the moment, which makes sense.
    idx: Cell<usize>,
    phantom: PhantomData<T>,
}

impl<T> PayloadEnumerator<T>
    where T: EndOffset
{
    /// Create a new iterator.
    #[inline]
    pub fn new(batch: &mut BatchIterator) -> PayloadEnumerator<T> {
        let start = batch.start();
        PayloadEnumerator {
            idx: Cell::new(start),
            phantom: PhantomData,
        }
    }

    /// Used for looping over packets. Note this iterator is not safe if packets are added or dropped during iteration,
    /// so you should not do that if possible.
    #[inline]
    pub fn next<'a>(&'a self, batch: &'a mut BatchIterator) -> Option<ParsedDescriptor<'a, T>> {
        let original_idx = self.idx.get();
        let item = unsafe { batch.next_payload(original_idx) };
        match item {
            Some((PacketDescriptor{offset, header: haddr, payload, payload_size},
                  ctx,
                  next_idx)) => {
                let header = cast_from_u8::<T>(haddr);
                // println!("Payload size is {}", payload_size);
                // This is safe (assuming our size accounting has been correct so far).
                let payload_slice = unsafe { from_raw_parts_mut::<u8>(payload, payload_size) };
                self.idx.set(next_idx);
                Some(ParsedDescriptor {
                    offset: offset,
                    index: original_idx,
                    header: header,
                    payload: payload_slice,
                    ctx: ctx,
                })
            }
            None => None,
        }
    }
}
