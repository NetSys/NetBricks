use common::*;
use interface::PortQueue;
use super::act::Act;
use super::Batch;
use super::iterator::{BatchIterator, PacketDescriptor};
use std::cmp;
use scheduler::Executable;
use headers::NullHeader;
use super::packet_batch::PacketBatch;

pub struct MergeBatch<T: Batch<Header=NullHeader>>
{
    parents: Vec<T>,
    which: usize,
}

impl<T: Batch<Header=NullHeader>> MergeBatch<T>
{
    pub fn new(parents: Vec<T>) -> MergeBatch<T> {
        MergeBatch {
            parents: parents,
            which: 0,
        }
    }
}

impl<T: Batch<Header=NullHeader>> Batch for MergeBatch<T>
{
}

impl<T: Batch<Header=NullHeader>> BatchIterator for MergeBatch<T>
{
    type Header = NullHeader;

    #[inline]
    fn start(&mut self) -> usize {
        self.parents[self.which].start()
    }

    #[inline]
    unsafe fn next_payload(&mut self, idx: usize) -> Option<PacketDescriptor<NullHeader>> {
        self.parents[self.which].next_payload(idx)
    }
}

/// Internal interface for packets.
impl<T: Batch<Header=NullHeader>> Act for MergeBatch<T>
{
    #[inline]
    fn act(&mut self) {
        self.parents[self.which].act()
    }

    #[inline]
    fn done(&mut self) {
        self.parents[self.which].done();
        // self.which = (self.which + 1) % self.parents.len();
        let next = self.which + 1;
        if next == self.parents.len() {
            self.which = 0
        } else {
            self.which = next
        }
    }

    #[inline]
    fn send_q(&mut self, port: &mut PortQueue) -> Result<u32> {
        self.parents[self.which].send_q(port)
    }

    #[inline]
    fn capacity(&self) -> i32 {
        self.parents.iter().fold(0, |acc, x| cmp::max(acc, x.capacity()))
    }

    #[inline]
    fn drop_packets(&mut self, idxes: &[usize]) -> Option<usize> {
        self.parents[self.which].drop_packets(idxes)
    }

    #[inline]
    fn clear_packets(&mut self) {
        self.parents[self.which].clear_packets()
    }

    #[inline]
    fn get_packet_batch(&mut self) -> &mut PacketBatch {
        self.parents[self.which].get_packet_batch()
    }
}

impl<T: Batch<Header=NullHeader>> Executable for MergeBatch<T>
{
    #[inline]
    fn execute(&mut self) {
        self.act();
        self.done();
    }
}
