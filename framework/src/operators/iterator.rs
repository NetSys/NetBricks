use std::marker::PhantomData;
use headers::EndOffset;
use std::cell::Cell;
use interface::Packet;

pub struct PacketDescriptor<T: EndOffset, M: Sized + Send> {
    pub packet: Packet<T, M>,
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
    type Header: EndOffset;
    type Metadata: Sized + Send;

    /// Returns the starting index for the packet batch. This allows for cases where the head of the batch is not at
    /// index 0.
    fn start(&mut self) -> usize;

    unsafe fn next_payload(&mut self, idx: usize) -> Option<PacketDescriptor<Self::Header, Self::Metadata>>;
}

/// A struct containing the parsed information returned by the `PayloadEnumerator`.
pub struct ParsedDescriptor<T, M>
    where T: EndOffset,
          M: Sized + Send
{
    pub index: usize,
    pub packet: Packet<T, M>,
}

/// An enumerator over both the header and the payload. The payload is represented as an appropriately sized slice of
/// bytes. The expectation is therefore that the user can operate on bytes, or make appropriate adjustments as
/// necessary.
pub struct PayloadEnumerator<T, M>
    where T: EndOffset,
          M: Sized + Send
{
    // Was originally using a cell here so we didn't have to borrow the iterator mutably. I think at this point, given
    // that the batch is not stored in the iterator this might be a moot point, but it does allow the iterator to be
    // entirely immutable for the moment, which makes sense.
    idx: Cell<usize>,
    _phantom_t: PhantomData<T>,
    _phantom_m: PhantomData<M>
}

impl<T, M> PayloadEnumerator<T, M>
    where T: EndOffset,
          M: Sized + Send
{
    /// Create a new iterator.
    #[inline]
    pub fn new(batch: &mut BatchIterator<Header = T, Metadata=M>) -> PayloadEnumerator<T, M> {
        let start = batch.start();
        PayloadEnumerator {
            idx: Cell::new(start),
            _phantom_t: PhantomData,
            _phantom_m: PhantomData,
        }
    }

    /// Used for looping over packets. Note this iterator is not safe if packets are added or dropped during iteration,
    /// so you should not do that if possible.
    #[inline]
    pub fn next(&self, batch: &mut BatchIterator<Header = T, Metadata=M>) -> Option<ParsedDescriptor<T, M>> {
        let original_idx = self.idx.get();
        let item = unsafe { batch.next_payload(original_idx) };
        match item {
            Some(PacketDescriptor { packet }) => {
                // This is safe (assuming our size accounting has been correct so far).
                // Switch to providing packets
                self.idx.set(original_idx + 1);
                Some(ParsedDescriptor {
                    index: original_idx,
                    packet: packet,
                })
            }
            None => None,
        }
    }
}
