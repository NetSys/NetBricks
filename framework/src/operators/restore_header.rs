use common::*;
use interface::*;
use headers::EndOffset;
use std::marker::PhantomData;
use super::act::Act;
use super::Batch;
use super::iterator::*;
use super::packet_batch::PacketBatch;

pub struct RestoreHeader<T, V>
    where T: EndOffset + 'static,
          V: Batch + BatchIterator + Act
{
    parent: V,
    phantom: PhantomData<T>,
}

impl<T, V> Act for RestoreHeader<T, V>
    where T: EndOffset + 'static,
          V: Batch + BatchIterator + Act
{
    act!{}
}

impl<T, V> Batch for RestoreHeader<T, V>
    where V: Batch + BatchIterator + Act,
          T: EndOffset + 'static
{
}

impl<T, V> RestoreHeader<T, V>
    where V: Batch + BatchIterator + Act,
          T: EndOffset + 'static
{
    #[inline]
    pub fn new(parent: V) -> RestoreHeader<T, V> {
        RestoreHeader {
            parent: parent,
            phantom: PhantomData,
        }
    }
}

impl<T, V> BatchIterator for RestoreHeader<T, V>
    where V: Batch + BatchIterator + Act,
          T: EndOffset + 'static
{
    type Header = T;
    unsafe fn next_payload(&mut self, idx: usize) -> Option<PacketDescriptor<T>> {
        self.parent.next_payload(idx).map(|p| PacketDescriptor { packet: p.packet.restore_saved_header().unwrap() })
    }

    #[inline]
    fn start(&mut self) -> usize {
        self.parent.start()
    }
}
