use super::act::Act;
use super::Batch;
use super::iterator::BatchIterator;
use io::PmdPort;
use io::Result;
use std::any::Any;

/// CompositionBatch allows multiple NFs to be combined. A composition batch resets the packet pointer so that each NF
/// can treat packets as originating from the NF itself.
pub struct CompositionBatch {
    parent: Box<Batch>,
}

impl CompositionBatch {
    pub fn new(parent: Box<Batch>) -> CompositionBatch {
        CompositionBatch { parent: parent }

    }
}

impl Batch for CompositionBatch {}

impl BatchIterator for CompositionBatch {
    #[inline]
    fn start(&mut self) -> usize {
        self.parent.start()
    }

    #[inline]
    unsafe fn next_address(&mut self, idx: usize, pop: i32) -> Option<(*mut u8, usize, Option<&mut Any>, usize)> {
        if pop != 0 {
            panic!("Cannot pop beyond a composition batch")
        }
        self.parent.next_base_address(idx)
    }

    #[inline]
    unsafe fn next_payload(&mut self, idx: usize) -> Option<(*mut u8, *mut u8, usize, Option<&mut Any>, usize)> {
        self.parent.next_base_payload(idx)
    }

    #[inline]
    unsafe fn next_base_address(&mut self, idx: usize) -> Option<(*mut u8, usize, Option<&mut Any>, usize)> {
        self.parent.next_base_address(idx)
    }

    #[inline]
    unsafe fn next_base_payload(&mut self, idx: usize) -> Option<(*mut u8, *mut u8, usize, Option<&mut Any>, usize)> {
        self.parent.next_base_payload(idx)
    }

    #[inline]
    unsafe fn next_payload_popped(&mut self,
                                  _: usize,
                                  _: i32)
                                  -> Option<(*mut u8, *mut u8, usize, Option<&mut Any>, usize)> {
        panic!("Cannot pop beyond a composition batch")
    }
}

/// Internal interface for packets.
impl Act for CompositionBatch {
    #[inline]
    fn act(&mut self) {
        self.parent.act();
    }

    #[inline]
    fn done(&mut self) {
        self.parent.done();
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
