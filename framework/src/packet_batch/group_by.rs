use headers::EndOffset;
use utils::{SpscConsumer,SpscProducer, new_spsc_queue};
use io::PortQueue;
use io::Result;
use super::act::Act;
use super::Batch;
use super::packet_batch::PacketBatch;
use super::iterator::*;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

pub type GroupFn<T> = Box<FnMut(&mut T, &mut [u8], Option<&mut Any>) -> usize>;
pub struct GroupBy<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator + Act
{
    parent: V,
    group_ct: usize,
    group_fn: GroupFn<T>,
    producers: Vec<SpscProducer>,
    consumers: HashMap<usize, SpscConsumer>,
}

impl<T, V> GroupBy<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator + Act
{
    pub fn new(parent: V, groups: usize, group_fn: GroupFn<T>) -> Arc<GroupBy<T, V>> {
        let cnt = parent.capacity();
        let (producers, consumers) = {
            let mut producers = Vec::with_capacity(groups);
            let mut consumers = HashMap::with_capacity(groups);
            for i in 0..groups {
                let (prod, consumer) = new_spsc_queue(1<<10).unwrap();
                producers.push(prod);
                consumers.insert(i, consumer);
            };
            (producers, consumers)
        };

        let mut obj = Arc::new(GroupBy {
            parent: parent,
            group_ct: groups,
            group_fn: group_fn,
            producers: producers,
            consumers: consumers,
        });
        obj
    }

    #[inline]
    pub fn group_count(&self) -> usize {
        self.group_ct
    }

}

pub struct GroupedBatch<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator + Act
{
    parent: PacketBatch,
    operator: Arc<GroupBy<T, V>>,
}

impl<T, V> GroupedBatch<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator + Act
{
    pub fn new(parent: PacketBatch, operator: Arc<GroupBy<T, V>>) -> GroupedBatch<T, V> {
        GroupedBatch {
            parent: parent,
            operator: operator,
        }

    }
}

impl<T, V> Batch for GroupedBatch<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator + Act
{
}

impl<T, V> BatchIterator for GroupedBatch<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator + Act
{
    #[inline]
    fn start(&mut self) -> usize {
        self.parent.start()
    }

    #[inline]
    unsafe fn next_payload(&mut self, idx: usize) -> Option<(PacketDescriptor, Option<&mut Any>, usize)> {
        self.parent.next_base_payload(idx)
    }

    #[inline]
    unsafe fn next_base_payload(&mut self, idx: usize) -> Option<(PacketDescriptor, Option<&mut Any>, usize)> {
        self.parent.next_base_payload(idx)
    }

    #[inline]
    unsafe fn next_payload_popped(&mut self,
                                  idx: usize,
                                  pop: i32)
                                  -> Option<(PacketDescriptor, Option<&mut Any>, usize)> {
        self.parent.next_payload_popped(idx, pop)
    }
}

/// Internal interface for packets.
impl<T, V> Act for GroupedBatch<T, V>
    where T: EndOffset,
          V: Batch + BatchIterator + Act
{
    act!{}
}
