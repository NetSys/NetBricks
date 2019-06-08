use super::{Batch, PacketError};
use crate::packets::{Packet, RawPacket};
use std::cell::RefCell;
use std::clone::Clone;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};

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

/// Alternative to receive operator as the start of a pipeline
///
/// Obtains the packets by dequeuing from an internal queue. Different
/// queue implementations enable various use cases.
///
/// * single threaded queue enables testing of the operators by making
/// packet ingest controllable through code.
/// * mpsc queue enables enqueuing packets from threads different from
/// the thread that sends them out.
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
        self.queue.dequeue().map(Ok)
    }

    #[inline]
    fn receive(&mut self) {
        // nop
    }
}

/// A reference counted VecDeque
pub struct SingleThreadedQueue<T>(Rc<RefCell<VecDeque<T>>>);

impl<T> SingleThreadedQueue<T> {
    #[inline]
    pub fn new(capacity: usize) -> Self {
        SingleThreadedQueue(Rc::new(RefCell::new(VecDeque::with_capacity(capacity))))
    }
}

impl<T> Clone for SingleThreadedQueue<T> {
    #[inline]
    fn clone(&self) -> Self {
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

/// Returns a `QueueBatch` and the corresponding producer for
/// single-threaded use.
///
/// The underlying `SingleThreadedQueue` can be cloned multiple times.
/// But it is only safe for single-threaded access. Only one concurrent
/// enqueue or dequeue operation can be called.
///
/// # Example
///
/// ```
/// let (producer, batch) = single_threaded_batch::<RawPacket>(10);
/// let mut batch = batch.map(mutate_packet);
///
/// // enqueue a packet using the producer
/// producer.enqueue(packet);
///
/// // push the packet through the pipeline
/// let new_packet = batch.next().unwrap();
/// ```
#[inline]
pub fn single_threaded_batch<T: Packet>(
    capacity: usize,
) -> (SingleThreadedQueue<T>, QueueBatch<SingleThreadedQueue<T>>) {
    let queue = SingleThreadedQueue::new(capacity);
    (queue.clone(), QueueBatch::new(queue))
}

/// A mpsc channel based producer
pub struct MpscProducer(Sender<RawPacket>);

impl MpscProducer {
    #[inline]
    pub fn new(sender: Sender<RawPacket>) -> Self {
        MpscProducer(sender)
    }
}

impl Clone for MpscProducer {
    #[inline]
    fn clone(&self) -> Self {
        MpscProducer(self.0.clone())
    }
}

impl Enqueue for MpscProducer {
    type Item = RawPacket;

    #[inline]
    fn enqueue(&self, item: Self::Item) {
        if self.0.send(item).is_err() {
            // Only way to get an error is if the receiver disconnected,
            // and if that happens, something is very wrong but no way
            // to recover from that. No more egress packets from this
            // particular queue.
            error!("The corresponding receiver for queue has disconnected");
        }
    }
}

impl Dequeue for Receiver<RawPacket> {
    type Item = RawPacket;

    fn dequeue(&self) -> Option<Self::Item> {
        match self.try_recv() {
            Ok(packet) => Some(packet),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                // Only way to get an error is if the sender disconnected,
                // and if that happens, something is very wrong but no way
                // to recover from that. No more egress packets from this
                // particular queue.
                error!("The corresponding sender for queue has disconnected");
                None
            }
        }
    }
}

/// Returns a mutliple producer, single consumer `QueueBatch`.
///
/// Based on rust mpsc channel. The producer can be cloned multiple times.
/// It is multi-thread safe. Both the enqueue and dequeue operations are
/// asynchronous and will not block.
///
/// Only `RawPacket` can be sent across thread boundaries.
///
/// # Example
///
/// ```
/// let (producer, batch) = mpsc_batch();
/// let mut batch = batch.map(mutate_packet);
///
/// thread::spawn(move || {
///     // enqueue a packet on a different thread
///     producer.enqueue(packet);
/// });
///
/// // receive the packet on main thread
/// let new_packet = batch.next().unwrap();
/// ```
#[inline]
pub fn mpsc_batch() -> (MpscProducer, QueueBatch<Receiver<RawPacket>>) {
    let (sender, receiver) = channel();
    (MpscProducer::new(sender), QueueBatch::new(receiver))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packets::{RawPacket, UDP_PACKET};

    #[test]
    fn single_threaded() {
        dpdk_test! {
            let (producer, mut batch) = single_threaded_batch::<RawPacket>(1);
            producer.enqueue(RawPacket::from_bytes(&UDP_PACKET).unwrap());

            assert!(batch.next().unwrap().is_ok());
        }
    }

    #[test]
    fn mpsc() {
        dpdk_test! {
            let (producer, mut batch) = mpsc_batch();
            producer.enqueue(RawPacket::from_bytes(&UDP_PACKET).unwrap());

            let thread = std::thread::spawn(move || {
                assert!(batch.next().unwrap().is_ok());
            });
            thread.join().unwrap()
        }
    }
}
