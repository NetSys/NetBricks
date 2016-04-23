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

// The actual queue.
struct SpscQueue {
    queue: Vec<AtomicPtr<MBuf>>,
    slots: usize,
    mask: usize,
    producer: QueuePlace,
    consumer: QueuePlace,
}

pub struct SpscConsumer {
    queue: Arc<SpscQueue>,
}

pub struct SpscProducer {
    queue: Arc<SpscQueue>,
}

/// Produce a producer and a consumer for a SPSC queue. The producer and consumer can be sent across a channel, however
/// they cannot be cloned.
pub fn new_spsc_queue(size: usize) -> Option<(SpscProducer, SpscConsumer)> {
    if (size & (size - 1)) == 0 {
        None
    } else {
        let q = Arc::new(SpscQueue::new(size).unwrap());
        Some((SpscProducer { queue: q.clone() }, SpscConsumer { queue: q.clone() }))
    }
}

impl SpscConsumer {
    /// Dequeue one mbuf from this queue.
    pub fn dequeue_one(&self) -> Option<*mut MBuf> {
        self.queue.dequeue_one()
    }

    /// Dequeue several mbufs from this queue.
    pub fn dequeue(&self, mbufs: &mut Vec<*mut MBuf>, cnt: usize) -> usize {
        self.queue.dequeue(mbufs, cnt)
    }
}

impl SpscProducer {
    /// Enqueue one mbuf in this queue.
    pub fn enqueue_one(&self, mbuf: *mut MBuf) -> bool {
        self.queue.enqueue_one(mbuf)
    }

    /// Enqueue several mbufs to this queue.
    pub fn enqueue(&self, mbufs: &mut Vec<*mut MBuf>) -> usize {
        self.queue.enqueue(mbufs)
    }
}

impl SpscQueue {
    /// Create a new SPSC queue. We require that the size of the ring be a power of two.
    pub fn new(slots: usize) -> Option<SpscQueue> {
        if (slots & (slots - 1)) == 0 {
            Some(SpscQueue {
                queue: (0..slots).map(|_| AtomicPtr::new(ptr::null_mut())).collect(),
                slots: slots,
                mask: slots - 1,
                producer: Default::default(),
                consumer: Default::default(),
            })
        } else {
            // Ring size must be a power of 2
            None
        }
    }

    pub fn enqueue_one(&self, mbuf: *mut MBuf) -> bool {
        let producer_head = self.producer.head.load(Ordering::Acquire);
        let consumer_tail = self.consumer.tail.load(Ordering::Acquire);
        let free_entries = self.mask + consumer_tail - producer_head;
        if free_entries == 0 {
            false
        } else {
            self.producer.head.store(producer_head + 1, Ordering::Release);
            let idx = producer_head & self.mask;
            self.queue[idx].store(mbuf, Ordering::Release);
            self.producer.tail.store(producer_head + 1, Ordering::Release);
            true
        }
    }

    pub fn enqueue(&self, mbufs: &mut Vec<*mut MBuf>) -> usize {
        let mask = self.mask;
        let producer_head = self.producer.head.load(Ordering::Acquire);
        let consumer_tail = self.consumer.tail.load(Ordering::Acquire);
        let free_entries = mask + consumer_tail - producer_head;
        let n = min(free_entries, mbufs.len());
        self.producer.head.store(producer_head + n, Ordering::Release);
        let mut idx = producer_head & mask;
        let slots = self.slots;
        if idx + n < slots {
            let unroll_end = n & (!0x3);
            let mut i = 0;
            while i < unroll_end {
                self.queue[idx].store(mbufs[i], Ordering::Relaxed);
                self.queue[idx + 1].store(mbufs[i + 1], Ordering::Relaxed);
                self.queue[idx + 2].store(mbufs[i + 2], Ordering::Relaxed);
                self.queue[idx + 3].store(mbufs[i + 3], Ordering::Relaxed);
                idx += 4;
                i += 4;
            }
            while i < n {
                self.queue[idx].store(mbufs[i], Ordering::Relaxed);
                idx += 1;
                i += 1;
            }
        } else {
            let mut count = 0;
            while idx < slots {
                self.queue[idx].store(mbufs[count], Ordering::Relaxed);
                idx += 1;
                count += 1;
            }
            idx = 0;
            while count < n {
                self.queue[idx].store(mbufs[count], Ordering::Relaxed);
                idx += 1;
                count += 1;
            }
        }
        self.producer.tail.store(producer_head + n, Ordering::Release);
        n
    }

    pub fn dequeue_one(&self) -> Option<*mut MBuf> {
        let consumer_head = self.consumer.head.load(Ordering::Acquire);
        let producer_tail = self.consumer.tail.load(Ordering::Acquire);
        let mask = self.mask;
        let available_entries = producer_tail - consumer_head;
        if available_entries == 0 {
            None
        } else {
            self.consumer.head.store(consumer_head + 1, Ordering::Release);
            let idx = consumer_head & mask;
            let val = Some(self.queue[idx].load(Ordering::Acquire));
            self.consumer.tail.store(consumer_head + 1, Ordering::Release);
            val
        }
    }

    pub fn dequeue(&self, mbufs: &mut Vec<*mut MBuf>, cnt: usize) -> usize {
        let consumer_head = self.consumer.head.load(Ordering::Acquire);
        let producer_tail = self.consumer.tail.load(Ordering::Acquire);
        let mask = self.mask;
        let slots = self.slots;
        let available_entries = producer_tail - consumer_head;
        let n = min(cnt, available_entries);
        let mut idx = consumer_head & mask;
        self.consumer.head.store(consumer_head + n, Ordering::Release);
        if idx + n < slots {
            let unroll_end = n & (!0x3);
            let mut i = 0;
            while i < unroll_end {
                mbufs.push(self.queue[idx].load(Ordering::Relaxed));
                mbufs.push(self.queue[idx + 1].load(Ordering::Relaxed));
                mbufs.push(self.queue[idx + 2].load(Ordering::Relaxed));
                mbufs.push(self.queue[idx + 3].load(Ordering::Relaxed));
                idx += 4;
                i += 4;
            }
            while i < n {
                mbufs.push(self.queue[idx].load(Ordering::Relaxed));
                idx += 1;
                i += 1;
            }
        } else {
            let mut i = 0;
            while idx < slots {
                mbufs.push(self.queue[idx].load(Ordering::Relaxed));
                idx += 1;
                i += 1;
            }
            idx = 0;
            while i < n {
                mbufs.push(self.queue[idx].load(Ordering::Relaxed));
            }
        }
        self.consumer.tail.store(consumer_head + n, Ordering::Release);
        n
    }
}
