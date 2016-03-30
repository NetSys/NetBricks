use io::PmdPort;
use io::Result;
use super::act::Act;
use super::Batch;
use super::CompositionBatch;
use super::iterator::{BatchIterator, PacketDescriptor};
use std::cmp;
use std::any::Any;

pub struct MergeBatch {
    parents: Vec<CompositionBatch>,
    which: usize,
}

impl MergeBatch {
    pub fn new(parents: Vec<CompositionBatch>) -> MergeBatch {
        MergeBatch {
            parents: parents,
            which: 0,
        }
    }

    #[inline]
    pub fn process(&mut self) {
        self.act();
        self.done();
    }
}

impl Batch for MergeBatch {}

impl BatchIterator for MergeBatch {
    #[inline]
    fn start(&mut self) -> usize {
        self.parents[self.which].start()
    }

    #[inline]
    unsafe fn next_payload(&mut self, idx: usize) -> Option<(PacketDescriptor, Option<&mut Any>, usize)> {
        self.parents[self.which].next_payload(idx)
    }

    #[inline]
    unsafe fn next_base_payload(&mut self, idx: usize) -> Option<(PacketDescriptor, Option<&mut Any>, usize)> {
        self.parents[self.which].next_base_payload(idx)
    }

    #[inline]
    unsafe fn next_payload_popped(&mut self,
                                  idx: usize,
                                  pop: i32)
                                  -> Option<(PacketDescriptor, Option<&mut Any>, usize)> {
        self.parents[self.which].next_payload_popped(idx, pop)
    }
}

/// Internal interface for packets.
impl Act for MergeBatch {
    #[inline]
    fn act(&mut self) {
        self.parents[self.which].act()
    }

    #[inline]
    fn done(&mut self) {
        self.parents[self.which].done();
        self.which = (self.which + 1) % self.parents.len();
    }

    #[inline]
    fn send_queue(&mut self, port: &mut PmdPort, queue: i32) -> Result<u32> {
        self.parents[self.which].send_queue(port, queue)
    }

    #[inline]
    fn capacity(&self) -> i32 {
        self.parents.iter().fold(0, |acc, x| cmp::max(acc, x.capacity()))
    }

    #[inline]
    fn drop_packets(&mut self, idxes: Vec<usize>) -> Option<usize> {
        self.parents[self.which].drop_packets(idxes)
    }

    #[inline]
    fn adjust_payload_size(&mut self, idx: usize, size: isize) -> Option<isize> {
        self.parents[self.which].adjust_payload_size(idx, size)
    }

    #[inline]
    fn adjust_headroom(&mut self, idx: usize, size: isize) -> Option<isize> {
        self.parents[self.which].adjust_headroom(idx, size)
    }
}
