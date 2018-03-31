use super::Batch;
use super::act::Act;
use super::iterator::*;
use super::packet_batch::PacketBatch;
use common::*;
use headers::EndOffset;
use interface::Packet;
use interface::PacketTx;

pub type FilterFn<T, M> = Box<FnMut(&Packet<T, M>) -> bool + Send>;

pub struct FilterBatch<T, V>
where
    T: EndOffset,
    V: Batch + BatchIterator<Header = T> + Act,
{
    parent: V,
    filter: FilterFn<T, V::Metadata>,
    capacity: usize,
    remove: Vec<usize>,
}

impl<T, V> FilterBatch<T, V>
where
    T: EndOffset,
    V: Batch + BatchIterator<Header = T> + Act,
{
    #[inline]
    pub fn new(parent: V, filter: FilterFn<T, V::Metadata>) -> FilterBatch<T, V> {
        let capacity = parent.capacity() as usize;
        FilterBatch {
            parent: parent,
            filter: filter,
            capacity: capacity,
            remove: Vec::with_capacity(capacity),
        }
    }
}

batch_no_new!{FilterBatch}

impl<T, V> Act for FilterBatch<T, V>
where
    T: EndOffset,
    V: Batch + BatchIterator<Header = T> + Act,
{
    #[inline]
    fn act(&mut self) {
        self.parent.act();
        // Filter during the act
        let iter = PayloadEnumerator::<T, V::Metadata>::new(&mut self.parent);
        while let Some(ParsedDescriptor { mut packet, index: idx }) = iter.next(&mut self.parent) {
            if !(self.filter)(&mut packet) {
                self.remove.push(idx)
            }
        }
        if !self.remove.is_empty() {
            self.parent
                .drop_packets(&self.remove[..])
                .expect("Filtering was performed incorrectly");
        }
        self.remove.clear();
    }

    #[inline]
    fn done(&mut self) {
        self.parent.done();
    }

    #[inline]
    fn send_q(&mut self, port: &PacketTx) -> Result<u32> {
        self.parent.send_q(port)
    }

    #[inline]
    fn capacity(&self) -> i32 {
        self.capacity as i32
    }

    #[inline]
    fn drop_packets(&mut self, idxes: &[usize]) -> Option<usize> {
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

impl<T, V> BatchIterator for FilterBatch<T, V>
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
