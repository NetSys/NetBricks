use super::{Batch, Enqueue, PacketError, QueueBatch, SingleThreadedQueue};
use packets::Packet;
use std::collections::HashMap;

pub type PipelineBuilder<T> = FnMut(QueueBatch<SingleThreadedQueue<T>>) -> Box<Batch<Item = T>>;

/// Lazily-evaluate group_by operator
///
/// When unmatched, the packet is marked as dropped and will short-circuit
/// the remainder of the pipeline.
///
/// On error, the packet is marked as aborted and will short-circuit the
/// remainder of the pipeline.
pub struct GroupByBatch<B: Batch, K, S>
where
    K: Eq + Clone + std::hash::Hash,
    S: FnMut(&B::Item) -> K,
{
    source: B,
    selector: S,
    producer: SingleThreadedQueue<B::Item>,
    groups: HashMap<K, Box<Batch<Item = B::Item>>>,
}

impl<B: Batch, K, S> GroupByBatch<B, K, S>
where
    K: Eq + Clone + std::hash::Hash,
    S: FnMut(&B::Item) -> K,
{
    #[inline]
    pub fn new<C>(source: B, selector: S, composer: C) -> Self
    where
        C: FnOnce(&mut HashMap<K, Box<PipelineBuilder<B::Item>>>) -> (),
    {
        let queue = SingleThreadedQueue::<B::Item>::new(1);
        let mut groups = HashMap::<K, Box<PipelineBuilder<B::Item>>>::new();
        composer(&mut groups);

        let groups = groups
            .iter_mut()
            .map(|(key, build)| {
                let key = key.clone();
                let group = build(QueueBatch::new(queue.clone()));
                (key, group)
            })
            .collect::<HashMap<_, _>>();

        GroupByBatch {
            source,
            selector,
            producer: queue,
            groups,
        }
    }
}

impl<B: Batch, K, S> Batch for GroupByBatch<B, K, S>
where
    K: Eq + Clone + std::hash::Hash,
    S: FnMut(&B::Item) -> K,
{
    type Item = B::Item;

    #[inline]
    fn next(&mut self) -> Option<Result<Self::Item, PacketError>> {
        self.source.next().map(|item| {
            match item {
                Ok(packet) => {
                    let key = (self.selector)(&packet);
                    match self.groups.get_mut(&key) {
                        Some(group) => {
                            self.producer.enqueue(packet);
                            group.next().unwrap()
                        }
                        // can't find the group, drop the packet
                        None => Err(PacketError::Drop(packet.mbuf())),
                    }
                }
                Err(e) => Err(e),
            }
        })
    }

    #[inline]
    fn receive(&mut self) {
        self.source.receive();
    }
}

/// Composes the pipelines for the group_by operator
#[macro_export]
macro_rules! compose {
    ($map:ident, $($key:expr => |$arg:tt| $body:block),*) => {{
        $(
            $map.insert($key, Box::new(|$arg| Box::new($body)));
        )*
    }}
}
