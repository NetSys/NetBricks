use super::act::Act;
use super::Batch;
use super::iterator::BatchIterator;
use super::super::pmd::*;
use super::super::interface::Result;

/// CompositionBatch allows multiple NFs to be combined. A composition batch resets the packet pointer so that each NF
/// can treat packets as originating from the NF itself.
pub struct CompositionBatch<V>
    where V: Batch + BatchIterator + Act
{
    parent: V,
}

impl<V> CompositionBatch<V>
    where V: Batch + BatchIterator + Act
{
    pub fn new(parent: V) -> CompositionBatch<V> {
        CompositionBatch { parent: parent }

    }
}

impl<V> Batch for CompositionBatch<V>
    where V: Batch + BatchIterator + Act
{
    type Parent = V;

    #[inline]
    fn pop(&mut self) -> &mut V {
        &mut self.parent
    }
}

impl<V> BatchIterator for CompositionBatch<V>
    where V: Batch + BatchIterator + Act
{
    #[inline]
    fn start(&mut self) -> usize {
        self.parent.start()
    }

    #[inline]
    unsafe fn next_address(&mut self, idx: usize) -> Option<(*mut u8, usize)> {
        self.parent.next_base_address(idx)
    }

    #[inline]
    unsafe fn next_payload(&mut self, idx: usize) -> Option<(*mut u8, usize)> {
        self.parent.next_base_payload(idx)
    }

    #[inline]
    unsafe fn next_base_address(&mut self, idx: usize) -> Option<(*mut u8, usize)> {
        self.parent.next_base_address(idx)
    }

    #[inline]
    unsafe fn next_base_payload(&mut self, idx: usize) -> Option<(*mut u8, usize)> {
        self.parent.next_base_payload(idx)
    }
}

/// Internal interface for packets.
impl<V> Act for CompositionBatch<V>
    where V: Batch + BatchIterator + Act
{
    #[inline]
    fn act(&mut self) -> &mut Self {
        self.parent.act();
        self
    }

    #[inline]
    fn done(&mut self) -> &mut Self {
        self.parent.done();
        self
    }

    #[inline]
    fn send_queue(&mut self, port: &mut PmdPort, queue: i32) -> Result<u32> {
        self.parent.send_queue(port, queue)
    }

    #[inline]
    fn capacity(&self) -> i32 {
        self.parent.capacity()
    }

    #[inline]
    fn drop_packets(&mut self, idxes: Vec<usize>) -> Option<usize> {
        self.parent.drop_packets(idxes)
    }
}
