use headers::EndOffset;
use utils::{SpscConsumer, SpscProducer, new_spsc_queue};
use super::act::Act;
use super::Batch;
use scheduler::{Executable, Scheduler};
use super::ReceiveQueue;
use super::iterator::*;
use std::any::Any;
use std::collections::HashMap;
use std::cell::RefCell;
use std::marker::PhantomData;

pub type GroupFn<T> = Box<FnMut(&mut T, &mut [u8], Option<&mut Any>) -> usize>;
struct GroupByProducer<T, V>
    where T: EndOffset + 'static,
          V: Batch + BatchIterator + Act + 'static
{
    parent: V,
    group_ct: usize,
    group_fn: GroupFn<T>,
    producers: Vec<SpscProducer>,
}
pub struct GroupBy<T, V>
    where T: EndOffset + 'static,
          V: Batch + BatchIterator + Act + 'static
{
    group_ct: usize,
    consumers: HashMap<usize, SpscConsumer>,
    phantom_t: PhantomData<T>,
    phantom_v: PhantomData<V>,
}

impl<T, V> GroupBy<T, V>
    where T: EndOffset + 'static,
          V: Batch + BatchIterator + Act + 'static
{
    pub fn new(parent: V, groups: usize, group_fn: GroupFn<T>, sched: &mut Scheduler) -> GroupBy<T, V> {
        let (producers, consumers) = {
            let mut producers = Vec::with_capacity(groups);
            let mut consumers = HashMap::with_capacity(groups);
            for i in 0..groups {
                let (prod, consumer) = new_spsc_queue(1 << 10).unwrap();
                producers.push(prod);
                consumers.insert(i, consumer);
            }
            (producers, consumers)
        };
        sched.add_task(RefCell::new(box GroupByProducer::<T, V> {
            parent: parent,
            group_ct: groups,
            group_fn: group_fn,
            producers:producers,
        }));
        GroupBy {
            group_ct: groups,
            consumers: consumers,
            phantom_t: PhantomData,
            phantom_v: PhantomData,
        }
    }

    #[inline]
    pub fn group_count(&self) -> usize {
        self.group_ct
    }

    #[inline]
    pub fn get_group(&mut self, group: usize) -> Option<ReceiveQueue> {
        // FIXME: This currently loses all the parsing, we should fix it to not be the case.
        if group > self.group_ct {
            None
        } else {
            self.consumers
                .remove(&group)
                .and_then(|q| Some(ReceiveQueue::new(q)))
        }
    }
}

impl<T, V> Executable for GroupByProducer<T, V>
    where T: EndOffset + 'static,
          V: Batch + BatchIterator + Act + 'static
{
    #[inline]
    fn execute(&mut self) {
        self.parent.act(); // Let the parent get some packets.
        {
            let iter = PayloadEnumerator::<T>::new(&mut self.parent);
            let mut groups = Vec::with_capacity(self.group_ct);
            while let Some(ParsedDescriptor { header: hdr, payload, ctx, index, .. }) = iter.next(&mut self.parent) {
                let group = (self.group_fn)(hdr, payload, ctx);
                groups.push((index, group));
            }
            // At this time groups contains what we need to distribute, so distribute it out.
            self.parent.distribute_to_queues(&self.producers, groups, true)
        }
    }
}
