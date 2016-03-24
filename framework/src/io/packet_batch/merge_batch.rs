use super::act::Act;
use super::Batch;
use super::CompositionBatch;
use super::iterator::BatchIterator;
use super::super::pmd::*;
use super::super::interface::Result;
use std::cmp;

pub struct MergeBatch
{
    parent1: CompositionBatch,
    parent2: CompositionBatch,
    which: i32,
}

impl MergeBatch 
{
    pub fn new(parent1: CompositionBatch, parent2: CompositionBatch) -> MergeBatch {
        MergeBatch {
            parent1: parent1,
            parent2: parent2,
            which: 0,
        }
    }
}

impl Batch for MergeBatch {}

impl BatchIterator for MergeBatch
{
    #[inline]
    fn start(&mut self) -> usize {
        match self.which {
            0 => self.parent1.start(),
            1 => self.parent2.start(),
            _ => panic!("Should not happen"),
        }
    }

    #[inline]
    unsafe fn next_address(&mut self, idx: usize) -> Option<(*mut u8, usize)> {
        match self.which {
            0 => self.parent1.next_address(idx),
            1 => self.parent2.next_address(idx),
            _ => panic!("Should not happen"),
        }
    }

    #[inline]
    unsafe fn next_payload(&mut self, idx: usize) -> Option<(*mut u8, usize)> {
        match self.which {
            0 => self.parent1.next_payload(idx),
            1 => self.parent2.next_payload(idx),
            _ => panic!("Should not happen"),
        }
    }

    #[inline]
    unsafe fn next_base_address(&mut self, idx: usize) -> Option<(*mut u8, usize)> {
        match self.which {
            0 => self.parent1.next_base_address(idx),
            1 => self.parent2.next_base_address(idx),
            _ => panic!("Should not happen"),
        }
    }

    #[inline]
    unsafe fn next_base_payload(&mut self, idx: usize) -> Option<(*mut u8, usize)> {
        match self.which {
            0 => self.parent1.next_base_payload(idx),
            1 => self.parent2.next_base_payload(idx),
            _ => panic!("Should not happen"),
        }
    }
}

/// Internal interface for packets.
impl Act for MergeBatch
{
    #[inline]
    fn act(&mut self) {
        match self.which {
            0 => { self.parent1.act(); () },
            1 => { self.parent2.act(); () },
            _ => panic!("Should not happen"),
        };
    }

    #[inline]
    fn done(&mut self) {
        match self.which {
            0 => { self.parent1.done(); () },
            1 => { self.parent2.done(); () },
            _ => panic!("Should not happen"),
        };
        self.which = (self.which + 1) % 2;
    }

    #[inline]
    fn send_queue(&mut self, port: &mut PmdPort, queue: i32) -> Result<u32> {
        match self.which {
            0 => self.parent1.send_queue(port, queue),
            1 => self.parent2.send_queue(port, queue),
            _ => panic!("Should not happen"),
        }
    }

    #[inline]
    fn capacity(&self) -> i32 {
        cmp::max(self.parent1.capacity(), self.parent2.capacity())
    }

    #[inline]
    fn drop_packets(&mut self, idxes: Vec<usize>) -> Option<usize> {
        match self.which {
            0 => self.parent1.drop_packets(idxes),
            1 => self.parent2.drop_packets(idxes),
            _ => panic!("Should not happen"),
        }
    }
}
