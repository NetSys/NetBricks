/// A single producer single consumer queue for mbufs. This is to do both `GroupBys` and Shuffles (which are really the
/// same thing in this case).
use io::*;
use std::ptr;
use std::cmp::min;
use std::default::Default;
use std::sync::Arc;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

#[derive(Default)]
struct QueuePlace {
    // FIXME: Actually don't need separate head and tail for the SPSC case.
    pub head: AtomicUsize,
    pub tail: AtomicUsize,
}

struct QueueElement<T>
    where T: 'static + Send + Sized
{
    pub addr: AtomicPtr<MBuf>,
    pub metadata: AtomicPtr<T>,
}

impl<T> Default for QueueElement<T>
    where T: 'static + Send + Sized
{
    fn default() -> QueueElement<T> {
        QueueElement {
            addr: AtomicPtr::new(ptr::null_mut()),
            metadata: AtomicPtr::new(ptr::null_mut()),
        }
    }
}

// The actual queue.
struct SpscQueue<T>
    where T: 'static + Send + Sized
{
    queue: Vec<QueueElement<T>>,
    slots: usize,
    mask: usize,
    producer: QueuePlace,
    consumer: QueuePlace,
}

pub struct SpscConsumer<T>
    where T: 'static + Send + Sized
{
    queue: Arc<SpscQueue<T>>,
}

pub struct SpscProducer<T>
    where T: 'static + Send + Sized
{
    queue: Arc<SpscQueue<T>>,
}

/// Produce a producer and a consumer for a SPSC queue. The producer and consumer can be sent across a channel, however
/// they cannot be cloned.
pub fn new_spsc_queue<T>(size: usize) -> Option<(SpscProducer<T>, SpscConsumer<T>)>
    where T: 'static + Send + Sized
{
    if (size & (size - 1)) == 0 {
        let q = Arc::new(SpscQueue::new(size).unwrap());
        Some((SpscProducer { queue: q.clone() }, SpscConsumer { queue: q.clone() }))
    } else {
        None
    }
}

impl<T> SpscConsumer<T>
    where T: 'static + Send + Sized
{
    /// Dequeue one mbuf from this queue.
    pub fn dequeue_one(&self) -> Option<(*mut MBuf, *mut T)> {
        self.queue.dequeue_one()
    }

    /// Dequeue several mbufs from this queue.
    pub fn dequeue(&self, mbufs: &mut Vec<*mut MBuf>, metas: &mut Vec<*mut T>, cnt: usize) -> usize {
        self.queue.dequeue(mbufs, metas, cnt)
    }
}

impl<T> SpscProducer<T>
    where T: 'static + Send + Sized
{
    /// Enqueue one mbuf in this queue.
    pub fn enqueue_one(&self, mbuf: *mut MBuf, meta: *mut T) -> bool {
        self.queue.enqueue_one(mbuf, meta)
    }

    /// Enqueue several mbufs to this queue.
    pub fn enqueue(&self, mbufs: &mut Vec<*mut MBuf>, metadata: &mut Vec<*mut T>) -> usize {
        self.queue.enqueue(mbufs, metadata)
    }
}


impl<T> SpscQueue<T>
    where T: 'static + Send + Sized
{
    /// Create a new SPSC queue. We require that the size of the ring be a power of two.
    pub fn new(slots: usize) -> Option<SpscQueue<T>> {
        if (slots & (slots - 1)) == 0 {
            Some(SpscQueue {
                queue: (0..slots).map(|_| Default::default()).collect(),
                slots: slots,
                mask: slots - 1,
                producer: Default::default(),
                consumer: Default::default(),
            })
        } else {
            None
        }
    }

    pub fn enqueue_one(&self, mbuf: *mut MBuf, metadata: *mut T) -> bool {
        let producer_head = self.producer.head.load(Ordering::Acquire);
        let consumer_tail = self.consumer.tail.load(Ordering::Acquire);
        let free_entries = self.mask.wrapping_add(consumer_tail).wrapping_sub(producer_head);
        if free_entries == 0 {
            false
        } else {
            let new_head = producer_head.wrapping_add(1);
            self.producer.head.store(new_head, Ordering::Release);
            let idx = producer_head & self.mask;
            self.queue[idx].addr.store(mbuf, Ordering::Relaxed);
            self.queue[idx].metadata.store(metadata, Ordering::Release);
            self.producer.tail.store(new_head, Ordering::Release);
            true
        }
    }

    pub fn enqueue(&self, mbufs: &mut Vec<*mut MBuf>, meta: &mut Vec<*mut T>) -> usize {
        let mask = self.mask;
        let producer_head = self.producer.head.load(Ordering::Acquire);
        let consumer_tail = self.consumer.tail.load(Ordering::Acquire);
        let free_entries = mask.wrapping_add(consumer_tail).wrapping_sub(producer_head);
        let n = min(free_entries, mbufs.len());
        let new_head = producer_head.wrapping_add(n);
        self.producer.head.store(new_head, Ordering::Release);
        let mut idx = producer_head & mask;
        let slots = self.slots;
        if idx + n < slots {
            let unroll_end = n & (!0x3);
            let mut i = 0;
            while i < unroll_end {
                self.queue[idx + 0].addr.store(mbufs[i], Ordering::Relaxed);
                self.queue[idx + 1].addr.store(mbufs[i + 1], Ordering::Relaxed);
                self.queue[idx + 2].addr.store(mbufs[i + 2], Ordering::Relaxed);
                self.queue[idx + 3].addr.store(mbufs[i + 3], Ordering::Relaxed);
                self.queue[idx + 0].metadata.store(meta[i], Ordering::Release);
                self.queue[idx + 1].metadata.store(meta[i + 1], Ordering::Release);
                self.queue[idx + 2].metadata.store(meta[i + 2], Ordering::Release);
                self.queue[idx + 3].metadata.store(meta[i + 3], Ordering::Release);
                idx += 4;
                i += 4;
            }
            while i < n {
                self.queue[idx].addr.store(mbufs[i], Ordering::Relaxed);
                self.queue[idx].metadata.store(meta[i], Ordering::Release);
                idx += 1;
                i += 1;
            }
        } else {
            let mut count = 0;
            while idx < slots {
                self.queue[idx].addr.store(mbufs[count], Ordering::Relaxed);
                self.queue[idx].metadata.store(meta[count], Ordering::Release);
                idx += 1;
                count += 1;
            }
            idx = 0;
            while count < n {
                self.queue[idx].addr.store(mbufs[count], Ordering::Relaxed);
                self.queue[idx].metadata.store(meta[count], Ordering::Release);
                idx += 1;
                count += 1;
            }
        }
        self.producer.tail.store(new_head, Ordering::Release);
        n
    }

    pub fn dequeue_one(&self) -> Option<(*mut MBuf, *mut T)> {
        let consumer_head = self.consumer.head.load(Ordering::Acquire);
        let producer_tail = self.producer.tail.load(Ordering::Acquire);
        let mask = self.mask;
        let available_entries = producer_tail.wrapping_sub(consumer_head);
        if available_entries == 0 {
            None
        } else {
            let new_consumer_head = consumer_head.wrapping_add(1);
            self.consumer.head.store(new_consumer_head, Ordering::Release);
            let idx = consumer_head & mask;
            let val = self.queue[idx].addr.load(Ordering::Acquire);
            let metadata_addr = self.queue[idx].metadata.load(Ordering::Acquire);
            self.consumer.tail.store(new_consumer_head, Ordering::Release);
            Some((val, metadata_addr))
        }
    }

    pub fn dequeue(&self, mbufs: &mut Vec<*mut MBuf>, meta: &mut Vec<*mut T>, cnt: usize) -> usize {
        let consumer_head = self.consumer.head.load(Ordering::Acquire);
        let producer_tail = self.producer.tail.load(Ordering::Acquire);
        let mask = self.mask;
        let slots = self.slots;
        let available_entries = producer_tail.wrapping_sub(consumer_head);
        let n = min(cnt, available_entries);
        let mut idx = consumer_head & mask;
        let new_head = consumer_head.wrapping_add(n);
        self.consumer.head.store(new_head, Ordering::Release);
        if idx + n < slots {
            let unroll_end = n & (!0x3);
            let mut i = 0;
            while i < unroll_end {
                mbufs.push(self.queue[idx + 0].addr.load(Ordering::Relaxed));
                mbufs.push(self.queue[idx + 1].addr.load(Ordering::Relaxed));
                mbufs.push(self.queue[idx + 2].addr.load(Ordering::Relaxed));
                mbufs.push(self.queue[idx + 3].addr.load(Ordering::Relaxed));
                meta.push(self.queue[idx + 0].metadata.load(Ordering::Relaxed));
                meta.push(self.queue[idx + 1].metadata.load(Ordering::Relaxed));
                meta.push(self.queue[idx + 2].metadata.load(Ordering::Relaxed));
                meta.push(self.queue[idx + 3].metadata.load(Ordering::Relaxed));
                idx += 4;
                i += 4;
            }
            while i < n {
                mbufs.push(self.queue[idx].addr.load(Ordering::Relaxed));
                meta.push(self.queue[idx].metadata.load(Ordering::Relaxed));
                idx += 1;
                i += 1;
            }
        } else {
            let mut i = 0;
            while idx < slots {
                mbufs.push(self.queue[idx].addr.load(Ordering::Relaxed));
                meta.push(self.queue[idx].metadata.load(Ordering::Relaxed));
                idx += 1;
                i += 1;
            }
            idx = 0;
            while i < n {
                mbufs.push(self.queue[idx].addr.load(Ordering::Relaxed));
                meta.push(self.queue[idx].metadata.load(Ordering::Relaxed));
                i += 1;
                idx += 1;
            }
        }
        self.consumer.tail.store(new_head, Ordering::Release);
        n
    }
}
