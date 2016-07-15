use common::*;
use super::iterator::*;
use super::act::Act;
use super::Batch;
use interface::PortQueue;
use interface::Packet;
use headers::EndOffset;
use super::packet_batch::PacketBatch;

pub type FilterFn<T> = Box<FnMut(&Packet<T>) -> bool + Send>;

pub struct FilterBatch<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator<Header=T> + Act
{
    parent: V,
    filter: FilterFn<T>,
    capacity: usize,
    remove: Vec<usize>,
}

impl<T, V> FilterBatch<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator<Header=T> + Act
{
    #[inline]
    pub fn new(parent: V, filter: FilterFn<T>) -> FilterBatch<T, V> {
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
    where T: EndOffset,
          V: Batch + BatchIterator<Header=T> + Act
{
    #[inline]
    fn act(&mut self) {
        self.parent.act();
        // let ref mut f = self.filter;
        let iter = PayloadEnumerator::<T>::new(&mut self.parent);
        while let Some(ParsedDescriptor { mut packet, index: idx }) = iter.next(&mut self.parent) {
            if (self.filter)(&mut packet) {
                self.remove.push(idx)
            }
        }
        if !self.remove.is_empty() {
            self.parent.drop_packets(&self.remove[..]).expect("Filtering was performed incorrectly");
        }
        self.remove.clear();
    }

    #[inline]
    fn done(&mut self) {
        self.parent.done();
    }

    #[inline]
    fn send_q(&mut self, port: &mut PortQueue) -> Result<u32> {
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
}

impl<T, V> BatchIterator for FilterBatch<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator<Header=T> + Act
{
    type Header = T;
    #[inline]
    fn start(&mut self) -> usize {
        self.parent.start()
    }

    #[inline]
    unsafe fn next_payload(&mut self, idx: usize) -> Option<PacketDescriptor<T>> {
        self.parent.next_payload(idx)
    }
}
