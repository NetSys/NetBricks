use super::act::Act;
use super::Batch;
use super::iterator::BatchIterator;
use super::super::pmd::*;
use super::super::interface::Result;
use std::default::Default;
use std::any::Any;

pub struct ContextBatch<T, V>
    where T: Any + Default + Clone,
          V: Batch + BatchIterator + Act
{
    parent: V,
    context: Vec<T>,
    context_size: usize,
}

impl<T, V> ContextBatch<T, V>
    where T: Any + Default + Clone,
          V: Batch + BatchIterator + Act
{
    pub fn new(parent: V) -> ContextBatch<T, V> {
        let capacity = parent.capacity() as usize;
        ContextBatch{
            parent: parent,
            context: vec![Default::default(); capacity],
            context_size: capacity,
        }
    }
}

impl<T, V> Act for ContextBatch<T, V>
    where T: Any + Default + Clone,
          V: Batch + BatchIterator + Act
{
    #[inline]
    fn act(&mut self) {
        self.parent.act();
    }

    #[inline]
    fn done(&mut self) {
        self.context.clear();
        self.context.resize(self.context_size, Default::default());
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

impl<T, V> BatchIterator for ContextBatch<T, V>
    where T: Any + Default + Clone,
          V: Batch + BatchIterator + Act
{
    #[inline]
    fn start(&mut self) -> usize {
        self.parent.start()
    }

    // FIXME: Really we should be accepting a token (capability) here and only adding context if the token matches.
    #[inline]
    unsafe fn next_address(&mut self, idx: usize) -> Option<(*mut u8, Option<&mut Any>, usize)> {
        match self.parent.next_address(idx) {
            Some((addr, _, idx)) => Some((addr, self.context.get_mut(idx).and_then(|x| Some(x as &mut Any)), idx)),
            None => None
        }
    }

    #[inline]
    unsafe fn next_payload(&mut self, idx: usize) -> Option<(*mut u8, Option<&mut Any>, usize)> {
        match self.parent.next_payload(idx) {
            Some((addr, _, idx)) => Some((addr, self.context.get_mut(idx).and_then(|x| Some(x as &mut Any)), idx)),
            None => None
        }
    }

    #[inline]
    unsafe fn next_base_address(&mut self, idx: usize) -> Option<(*mut u8, Option<&mut Any>, usize)> {
        match self.parent.next_base_address(idx) {
            Some((addr, _, idx)) => Some((addr, self.context.get_mut(idx).and_then(|x| Some(x as &mut Any)), idx)),
            None => None
        }
    }

    #[inline]
    unsafe fn next_base_payload(&mut self, idx: usize) -> Option<(*mut u8, Option<&mut Any>, usize)> {
        match self.parent.next_base_payload(idx) {
            Some((addr, _, idx)) => Some((addr, self.context.get_mut(idx).and_then(|x| Some(x as &mut Any)), idx)),
            None => None
        }
    }
}
