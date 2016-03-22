use super::iterator::{BatchIterator, PacketBatchAddressIterator};
use super::act::Act;
use super::Batch;
use super::packet_batch::cast_from_u8;
use super::super::interface::EndOffset;
use super::super::interface::Result;
use super::super::pmd::*;

pub struct TransformBatch<'a, T, V>
    where T: 'a + EndOffset,
          V: 'a + Batch + BatchIterator + Act
{
    parent: V,
    transformer: &'a mut FnMut(&'a mut T),
}

impl<'a, T, V> TransformBatch<'a, T, V>
    where T: 'a + EndOffset,
          V: 'a + Batch + BatchIterator + Act
{
    #[inline]
    pub fn new(parent: V, transformer: &'a mut FnMut(&'a mut T)) -> TransformBatch<'a, T, V> {
        TransformBatch {
            parent: parent,
            transformer: transformer,
        }
    }
}

impl<'a, T, V> Batch for TransformBatch<'a, T, V>
    where T: 'a + EndOffset,
          V: 'a + Batch + BatchIterator + Act
{
    type Parent = V;
    type Header = T;

    fn pop(&mut self) -> &mut V {
        &mut self.parent
    }
}

impl<'a, T, V> Act for TransformBatch<'a, T, V>
    where T: 'a + EndOffset,
          V: 'a + Batch + BatchIterator + Act
{
    fn act(&mut self) -> &mut Self {
        self.parent.act();
        {
            let ref mut f = self.transformer;
            let iter = PacketBatchAddressIterator::new(&mut self.parent);
            for addr in iter {
                let address = cast_from_u8::<T>(addr);
                f(address);
            }
        }
        self
    }

    fn done(&mut self) -> &mut Self {
        self.parent.done();
        self
    }

    fn send_queue(&mut self, port: &mut PmdPort, queue: i32) -> Result<u32> {
        self.parent.send_queue(port, queue)
    }

    fn capacity(&self) -> i32 {
        self.parent.capacity()
    }
}

impl<'a, T, V> BatchIterator for TransformBatch<'a, T, V>
    where T: 'a + EndOffset,
          V: 'a + Batch + BatchIterator + Act
{
    #[inline]
    fn start(&mut self) -> usize {
        self.parent.start()
    }

    #[inline]
    unsafe fn payload(&mut self, idx: usize) -> *mut u8 {
        self.parent.payload(idx)
    }

    #[inline]
    unsafe fn address(&mut self, idx: usize) -> *mut u8 {
        self.parent.address(idx)
    }

    #[inline]
    unsafe fn next_address(&mut self, idx: usize) -> Option<(*mut u8, usize)> {
        self.parent.next_address(idx)
    }

    #[inline]
    unsafe fn next_payload(&mut self, idx: usize) -> Option<(*mut u8, usize)> {
        self.parent.next_payload(idx)
    }

    #[inline]
    unsafe fn base_address(&mut self, idx: usize) -> *mut u8 {
        self.parent.base_address(idx)
    }

    #[inline]
    unsafe fn base_payload(&mut self, idx: usize) -> *mut u8 {
        self.parent.base_payload(idx)
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
