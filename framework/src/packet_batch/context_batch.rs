use io::PortQueue;
use io::Result;
use super::act::Act;
use super::Batch;
use super::HeaderOperations;
use super::iterator::{BatchIterator, PacketDescriptor};
use std::default::Default;
use std::any::Any;

pub struct ContextBatch<T, V>
    where T: 'static + Any + Default + Clone + Sized + Send,
          V: Batch + BatchIterator + Act
{
    parent: V,
    context: Vec<T>,
    _context_size: usize,
}

impl<T, V> ContextBatch<T, V>
    where T: 'static + Any + Default + Clone + Sized + Send,
          V: Batch + BatchIterator + Act
{
    pub fn new(parent: V) -> ContextBatch<T, V> {
        let capacity = parent.capacity() as usize;
        ContextBatch {
            parent: parent,
            context: vec![Default::default(); capacity],
            _context_size: capacity,
        }
    }
}

impl<T, V> Batch for ContextBatch<T, V>
    where T: 'static + Any + Default + Clone + Sized + Send,
          V: Batch + BatchIterator + Act
{
}

impl<T, V> HeaderOperations for ContextBatch<T, V>
   where T: 'static + Any + Default + Clone + Sized + Send,
         V: Batch + BatchIterator + Act + HeaderOperations
{
    type Header = V::Header; 
}


impl<T, V> Act for ContextBatch<T, V>
    where T: 'static + Any + Default + Clone + Sized + Send,
          V: Batch + BatchIterator + Act
{
    #[inline]
    fn parent(&mut self) -> &mut Batch {
        &mut self.parent
    }

    #[inline]
    fn parent_immutable(&self) -> &Batch {
        &self.parent
    }
    #[inline]
    fn act(&mut self) {
        self.parent.act();
    }

    #[inline]
    fn done(&mut self) {
        // FIXME: Cheaply zero out the vec.
        self.parent.done();
    }

    #[inline]
    fn send_q(&mut self, port: &mut PortQueue) -> Result<u32> {
        self.parent.send_q(port)
    }

    #[inline]
    fn capacity(&self) -> i32 {
        self.parent.capacity()
    }

    #[inline]
    fn drop_packets(&mut self, idxes: Vec<usize>) -> Option<usize> {
        // Need to adjust data
        let mut idx_orig = self.parent.start();
        let mut idx_new = 0;
        let mut remove_idx = 0;
        let end = self.context.len();

        // First go through the list of indexes to be filtered and get rid of them.
        while idx_orig < end && (remove_idx < idxes.len()) {
            let test_idx = idxes[remove_idx];
            assert!(idx_orig <= test_idx);
            if idx_orig == test_idx {
                remove_idx += 1;
            } else {
                self.context.swap(idx_orig, idx_new);
                idx_new += 1;
            }
            idx_orig += 1;
        }
        // Then copy over any left over packets.
        while idx_orig < end {
            self.context.swap(idx_orig, idx_new);
            idx_orig += 1;
            idx_new += 1;
        }
        self.parent.drop_packets(idxes)
    }

    #[inline]
    fn adjust_payload_size(&mut self, idx: usize, size: isize) -> Option<isize> {
        self.parent.adjust_payload_size(idx, size)
    }

    #[inline]
    fn adjust_headroom(&mut self, idx: usize, size: isize) -> Option<isize> {
        self.parent.adjust_headroom(idx, size)
    }
}

impl<T, V> BatchIterator for ContextBatch<T, V>
    where T: 'static + Any + Default + Clone + Sized + Send,
          V: Batch + BatchIterator + Act
{
    #[inline]
    fn start(&mut self) -> usize {
        self.parent.start()
    }

    #[inline]
    unsafe fn next_payload(&mut self, idx: usize) -> Option<(PacketDescriptor, Option<&mut Any>, usize)> {
        match self.parent.next_payload(idx) {
            Some((descriptor, _, iret)) => {
                Some((descriptor, self.context.get_mut(idx).and_then(|x| Some(x as &mut Any)), iret))
            }
            None => None,
        }
    }

    #[inline]
    unsafe fn next_base_payload(&mut self, idx: usize) -> Option<(PacketDescriptor, Option<&mut Any>, usize)> {
        match self.parent.next_base_payload(idx) {
            Some((descriptor, _, iret)) => {
                Some((descriptor, self.context.get_mut(idx).and_then(|x| Some(x as &mut Any)), iret))
            }
            None => None,
        }
    }

    #[inline]
    unsafe fn next_payload_popped(&mut self,
                                  idx: usize,
                                  pop: i32)
                                  -> Option<(PacketDescriptor, Option<&mut Any>, usize)> {
        match self.parent.next_payload_popped(idx, pop) {
            Some((descriptor, _, iret)) => {
                Some((descriptor, self.context.get_mut(idx).and_then(|x| Some(x as &mut Any)), iret))
            }
            None => None,
        }
    }
}
