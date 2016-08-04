use headers::EndOffset;
use queues::*;
use super::act::Act;
use super::Batch;
use scheduler::{Executable, Scheduler};
use super::ReceiveQueueGen;
use interface::Packet;
use super::iterator::*;
use std::collections::HashMap;
use std::marker::PhantomData;

// FIXME: This is moving all the metadata stuff int
pub type GroupFn<T> = Box<FnMut(&Packet<T>) -> usize + Send>;

pub struct GroupBy<T, V>
    where T: EndOffset + 'static,
          V: Batch + BatchIterator<Header = T> + Act + 'static
{
    _phantom_v: PhantomData<V>,
    groups: usize,
    _phantom_t: PhantomData<T>,
    consumers: HashMap<usize, ReceiveQueueGen<MpscConsumer>>,
}

struct GroupByProducer<T, V>
    where T: EndOffset + 'static,
          V: Batch + BatchIterator<Header = T> + Act + 'static
{
    parent: V,
    producers: Vec<MpscProducer>,
    group_fn: GroupFn<T>,
}

impl<T, V> Executable for GroupByProducer<T, V>
    where T: EndOffset + 'static,
          V: Batch + BatchIterator<Header = T> + Act + 'static
{
    #[inline]
    fn execute(&mut self) {
        self.parent.act(); // Let the parent get some packets.
        {
            let iter = PayloadEnumerator::<T>::new(&mut self.parent);
            while let Some(ParsedDescriptor { packet, .. }) = iter.next(&mut self.parent) {
                let group = (self.group_fn)(&packet);
                self.producers[group].enqueue_one(packet);
            }
        }
        self.parent.get_packet_batch().clear_packets();
        self.parent.done();
    }
}

impl<T, V> GroupBy<T, V>
    where T: EndOffset + 'static,
          V: Batch + BatchIterator<Header = T> + Act + 'static
{
    pub fn new(parent: V, groups: usize, group_fn: GroupFn<T>, sched: &mut Scheduler) -> GroupBy<T, V> {
        let mut producers = Vec::with_capacity(groups);
        let mut consumers = HashMap::with_capacity(groups);
        for i in 0..groups {
            let (prod, consumer) = new_mpsc_queue_pair();
            producers.push(prod);
            consumers.insert(i, consumer);
        }
        sched.add_task(GroupByProducer {
            parent: parent,
            group_fn: group_fn,
            producers: producers,
        });
        GroupBy {
            _phantom_v: PhantomData,
            groups: groups,
            _phantom_t: PhantomData,
            consumers: consumers,
        }
    }

    pub fn len(&self) -> usize {
        self.groups
    }

    pub fn get_group(&mut self, group: usize) -> Option<ReceiveQueueGen<MpscConsumer>> {
        self.consumers.remove(&group)
    }
}
