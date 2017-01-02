use headers::EndOffset;
use interface::Packet;
use queues::*;
use scheduler::{Executable, Scheduler};
use std::collections::HashMap;
use std::marker::PhantomData;
use super::Batch;
use super::ReceiveBatch;
use super::RestoreHeader;
use super::act::Act;
use super::iterator::*;

pub type GroupFn<T, M> = Box<FnMut(&Packet<T, M>) -> usize + Send>;

pub struct GroupBy<T, V>
    where T: EndOffset + 'static,
          V: Batch + BatchIterator<Header = T> + Act + 'static
{
    _phantom_v: PhantomData<V>,
    groups: usize,
    _phantom_t: PhantomData<T>,
    consumers: HashMap<usize, ReceiveBatch<MpscConsumer>>,
}

struct GroupByProducer<T, V>
    where T: EndOffset + 'static,
          V: Batch + BatchIterator<Header = T> + Act + 'static
{
    parent: V,
    producers: Vec<MpscProducer>,
    group_fn: GroupFn<T, V::Metadata>,
}

impl<T, V> Executable for GroupByProducer<T, V>
    where T: EndOffset + 'static,
          V: Batch + BatchIterator<Header = T> + Act + 'static
{
    #[inline]
    fn execute(&mut self) {
        self.parent.act(); // Let the parent get some packets.
        {
            let iter = PayloadEnumerator::<T, V::Metadata>::new(&mut self.parent);
            while let Some(ParsedDescriptor { mut packet, .. }) = iter.next(&mut self.parent) {
                let group = (self.group_fn)(&packet);
                packet.save_header_and_offset();
                self.producers[group].enqueue_one(packet);
            }
        }
        self.parent.get_packet_batch().clear_packets();
        self.parent.done();
    }
}

#[cfg_attr(feature = "dev", allow(len_without_is_empty))]
impl<T, V> GroupBy<T, V>
    where T: EndOffset + 'static,
          V: Batch + BatchIterator<Header = T> + Act + 'static
{
    pub fn new(parent: V, groups: usize, group_fn: GroupFn<T, V::Metadata>, sched: &mut Scheduler) -> GroupBy<T, V> {
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

    pub fn get_group(&mut self, group: usize) -> Option<RestoreHeader<T, V::Metadata, ReceiveBatch<MpscConsumer>>> {
        self.consumers.remove(&group).map(RestoreHeader::new)
    }
}
