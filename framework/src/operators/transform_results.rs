use super::act::Act;
use super::iterator::*;
use super::packet_batch::PacketBatch;
use super::Batch;
use common::*;
use headers::EndOffset;
use interface::Packet;
use interface::PacketTx;
use std::marker::PhantomData;

pub type TransformFnResult<T, M> = Box<FnMut(&mut Packet<T, M>) -> Result<()> + Send>;

pub struct TransformResults<T, V>
where
    T: EndOffset,
    V: Batch + BatchIterator<Header = T> + Act,
{
    parent: V,
    transformer: TransformFnResult<T, V::Metadata>,
    applied: bool,
    remove: Vec<usize>,
    phantom_t: PhantomData<T>,
}

impl<T, V> TransformResults<T, V>
where
    T: EndOffset,
    V: Batch + BatchIterator<Header = T> + Act,
{
    pub fn new(
        parent: V,
        transformer: TransformFnResult<T, V::Metadata>,
    ) -> TransformResults<T, V> {
        let capacity = parent.capacity() as usize;
        TransformResults {
            parent: parent,
            transformer: transformer,
            applied: false,
            remove: Vec::with_capacity(capacity),
            phantom_t: PhantomData,
        }
    }
}

impl<T, V> Batch for TransformResults<T, V>
where
    T: EndOffset,
    V: Batch + BatchIterator<Header = T> + Act,
{
}

impl<T, V> BatchIterator for TransformResults<T, V>
where
    T: EndOffset,
    V: Batch + BatchIterator<Header = T> + Act,
{
    type Header = T;
    type Metadata = <V as BatchIterator>::Metadata;
    #[inline]
    fn start(&mut self) -> usize {
        self.parent.start()
    }

    #[inline]
    unsafe fn next_payload(&mut self, idx: usize) -> Option<PacketDescriptor<T, Self::Metadata>> {
        self.parent.next_payload(idx)
    }
}

impl<T, V> Act for TransformResults<T, V>
where
    T: EndOffset,
    V: Batch + BatchIterator<Header = T> + Act,
{
    #[inline]
    fn act(&mut self) {
        if !self.applied {
            self.parent.act();
            {
                let iter = PayloadEnumerator::<T, V::Metadata>::new(&mut self.parent);
                while let Some(ParsedDescriptor {
                    mut packet,
                    index: idx,
                }) = iter.next(&mut self.parent)
                {
                    if let Err(ref e) = (self.transformer)(&mut packet) {
                        error_chain!(e);
                        self.remove.push(idx)
                    }
                }
            }

            self.applied = true;

            if !self.remove.is_empty() {
                self.parent
                    .drop_packets(&self.remove[..])
                    .map_or_else(|ref e| error_chain!(e), |_| ())
            }
            self.remove.clear();
        }
    }

    #[inline]
    fn done(&mut self) {
        self.applied = false;
        self.parent.done();
    }

    #[inline]
    fn send_q(&mut self, port: &PacketTx) -> Result<u32> {
        self.parent.send_q(port)
    }

    #[inline]
    fn capacity(&self) -> i32 {
        self.parent.capacity()
    }

    #[inline]
    fn drop_packets(&mut self, idxes: &[usize]) -> Result<usize> {
        self.parent.drop_packets(idxes)
    }

    #[inline]
    fn clear_packets(&mut self) {
        self.parent.clear_packets()
    }

    #[inline]
    fn get_packet_batch(&mut self) -> &mut PacketBatch {
        self.parent.get_packet_batch()
    }

    #[inline]
    fn get_task_dependencies(&self) -> Vec<usize> {
        self.parent.get_task_dependencies()
    }
}
