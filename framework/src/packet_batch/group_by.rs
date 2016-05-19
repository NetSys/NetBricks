use headers::EndOffset;
use utils::{SpscConsumer, SpscProducer, new_spsc_queue};
use super::act::Act;
use super::Batch;
use scheduler::{Executable, Scheduler};
use super::ReceiveQueue;
use super::iterator::*;
use std::any::*;
use std::collections::HashMap;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::ptr;

pub type GroupFn<T, S> = Box<FnMut(&mut T, &mut [u8], Option<&mut Any>) -> (usize, Option<Box<S>>) + Send>;

struct GroupByProducer<T, V, S>
    where T: EndOffset + 'static,
          V: Batch + BatchIterator + Act + 'static,
          S: 'static + Any + Default + Clone + Sized + Send
{
    parent: V,
    capacity: usize,
    group_ct: usize,
    group_fn: GroupFn<T, S>,
    producers: Vec<SpscProducer<u8>>,
}
pub struct GroupBy<T, V, S>
    where T: EndOffset + 'static,
          V: Batch + BatchIterator + Act + 'static,
          S: 'static + Any + Default + Clone + Sized + Send
{
    group_ct: usize,
    consumers: HashMap<usize, SpscConsumer<u8>>,
    phantom_t: PhantomData<T>,
    phantom_v: PhantomData<V>,
    phantom_s: PhantomData<S>,
}

impl<T, V, S> GroupBy<T, V, S>
    where T: EndOffset + 'static,
          V: Batch + BatchIterator + Act + 'static,
          S: 'static + Any + Default + Clone + Sized + Send
{
    pub fn new(parent: V, groups: usize, group_fn: GroupFn<T, S>, sched: &mut Scheduler) -> GroupBy<T, V, S> {
        let capacity = parent.capacity() as usize;
        let (producers, consumers) = {
            let mut producers = Vec::with_capacity(groups);
            let mut consumers = HashMap::with_capacity(groups);
            for i in 0..groups {
                let (prod, consumer) = new_spsc_queue(1 << 20).unwrap();
                producers.push(prod);
                consumers.insert(i, consumer);
            }
            (producers, consumers)
        };
        sched.add_task(RefCell::new(box GroupByProducer::<T, V, S> {
            parent: parent,
            capacity: capacity,
            group_ct: groups,
            group_fn: group_fn,
            producers: producers,
        }));
        GroupBy {
            group_ct: groups,
            consumers: consumers,
            phantom_t: PhantomData,
            phantom_v: PhantomData,
            phantom_s: PhantomData,
        }
    }

    #[inline]
    pub fn group_count(&self) -> usize {
        self.group_ct
    }

    #[inline]
    pub fn get_group(&mut self, group: usize) -> Option<ReceiveQueue<S>> {
        // FIXME: This currently loses all the parsing, we should fix it to not be the case.
        if group > self.group_ct {
            None
        } else {
            self.consumers
                .remove(&group)
                .and_then(|q| Some(ReceiveQueue::<S>::new(q)))
        }
    }
}

impl<T, V, S> Executable for GroupByProducer<T, V, S>
    where T: EndOffset + 'static,
          V: Batch + BatchIterator + Act + 'static,
          S: 'static + Any + Default + Clone + Sized + Send
{
    #[inline]
    fn execute(&mut self) {
        self.parent.act(); // Let the parent get some packets.
        {
            let iter = PayloadEnumerator::<T>::new(&mut self.parent);
            let mut groups = Vec::with_capacity(self.capacity);
            while let Some(ParsedDescriptor { header: hdr, payload, ctx, .. }) = iter.next(&mut self.parent) {
                let (group, meta) = (self.group_fn)(hdr, payload, ctx);
                groups.push((group,
                             meta.and_then(|m| Some(Box::into_raw(m) as *mut u8))
                                 .unwrap_or_else(|| ptr::null_mut())))
            }
            // At this time groups contains what we need to distribute, so distribute it out.
            self.parent.distribute_to_queues(&self.producers, &groups, self.group_ct)
        };
        self.parent.done();
    }
}
