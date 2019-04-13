use super::{Batch, PacketError};
use packets::Packet;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

/// A type that can enqueue items
pub trait Enqueue {
    type Item;

    fn enqueue(&self, item: Self::Item);
}

/// A type that can dequeue items
pub trait Dequeue {
    type Item;

    fn dequeue(&self) -> Option<Self::Item>;
}

/// A reference counted VecQueue
///
/// Only safe for single-threaded access
pub struct SingleThreadedQueue<T>(Rc<RefCell<VecDeque<T>>>);

impl<T> SingleThreadedQueue<T> {
    #[inline]
    pub fn new(capacity: usize) -> Self {
        SingleThreadedQueue(Rc::new(RefCell::new(VecDeque::with_capacity(capacity))))
    }

    #[inline]
    pub fn clone(&self) -> Self {
        SingleThreadedQueue(self.0.clone())
    }
}

impl<T> Enqueue for SingleThreadedQueue<T> {
    type Item = T;

    #[inline]
    fn enqueue(&self, item: Self::Item) {
        self.0.borrow_mut().push_back(item);
    }
}

impl<T> Dequeue for SingleThreadedQueue<T> {
    type Item = T;

    #[inline]
    fn dequeue(&self) -> Option<Self::Item> {
        self.0.borrow_mut().pop_front()
    }
}

/// Queue based receive operator
///
/// New items can be enqueued through the producer reference. To
/// create a new instance, use `single_threaded_batch` function.
pub struct QueueBatch<Q: Dequeue> {
    queue: Q,
}

impl<Q: Dequeue> QueueBatch<Q> {
    #[inline]
    pub fn new(queue: Q) -> Self {
        QueueBatch { queue }
    }
}

impl<Q: Dequeue> Batch for QueueBatch<Q>
where
    Q::Item: Packet,
{
    type Item = Q::Item;

    #[inline]
    fn next(&mut self) -> Option<Result<Self::Item, PacketError>> {
        self.queue.dequeue().map(|p| Ok(p))
    }

    #[inline]
    fn receive(&mut self) {
        // nop
    }
}

/// Returns a `QueueBatch` and the corresponding producer for
/// single-threaded use.
#[inline]
pub fn single_threaded_batch<T: Packet>(
    capacity: usize,
) -> (SingleThreadedQueue<T>, QueueBatch<SingleThreadedQueue<T>>) {
    let queue = SingleThreadedQueue::new(capacity);
    (queue.clone(), QueueBatch::new(queue))
}
