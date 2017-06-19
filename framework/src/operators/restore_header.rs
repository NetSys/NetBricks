use super::Batch;
use super::act::Act;
use super::iterator::*;
use super::packet_batch::PacketBatch;
use common::*;
use headers::EndOffset;
use interface::*;
use std::marker::PhantomData;

pub struct RestoreHeader<T, M, V>
where
    T: EndOffset + 'static,
    M: Sized + Send,
    V: Batch + BatchIterator + Act,
{
    parent: V,
    _phantom_t: PhantomData<T>,
    _phantom_m: PhantomData<M>,
}

impl<T, M, V> Act for RestoreHeader<T, M, V>
where
    T: EndOffset + 'static,
    M: Sized + Send,
    V: Batch + BatchIterator + Act,
{
    act!{}
}

impl<T, M, V> Batch for RestoreHeader<T, M, V>
where
    V: Batch + BatchIterator + Act,
    M: Sized + Send,
    T: EndOffset + 'static,
{
}

impl<T, M, V> RestoreHeader<T, M, V>
where
    V: Batch + BatchIterator + Act,
    M: Sized + Send,
    T: EndOffset + 'static,
{
    #[inline]
    pub fn new(parent: V) -> RestoreHeader<T, M, V> {
        RestoreHeader {
            parent: parent,
            _phantom_t: PhantomData,
            _phantom_m: PhantomData,
        }
    }
}

impl<T, M, V> BatchIterator for RestoreHeader<T, M, V>
where
    V: Batch + BatchIterator + Act,
    M: Sized + Send,
    T: EndOffset + 'static,
{
    type Header = T;
    type Metadata = M;
    unsafe fn next_payload(&mut self, idx: usize) -> Option<PacketDescriptor<T, M>> {
        self.parent.next_payload(idx).map(|p| {
            PacketDescriptor { packet: p.packet.restore_saved_header().unwrap() }
        })
    }

    #[inline]
    fn start(&mut self) -> usize {
        self.parent.start()
    }
}
