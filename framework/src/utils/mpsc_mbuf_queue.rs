/// A multiproducer single consumer queue for mbufs. The main difference when compared to `std::sync::mpsc` is that this
/// does not use a linked list (to avoid allocation). The hope is to eventually turn this into something that can carry
/// `Packets` or sufficient metadata to reconstruct that structure.
use io::*;
use interface::Packet;
use headers::EndOffset;
use packet_batch::ReceiveQueueGen;
use std::cmp::min;
use std::default::Default;
use std::sync::Arc;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use super::round_to_power_of_2;
use super::pause;
use super::ReceivableQueue;

#[derive(Default)]
struct QueueMetadata {
    pub head: AtomicUsize,
    pub tail: AtomicUsize,
}

struct MpscQueue {
    slots: usize, // Must be a power of 2
    mask: usize, // slots - 1
    // FIXME: Watermark?
    producer: QueueMetadata,
    consumer: QueueMetadata,
    queue: Vec<AtomicPtr<MBuf>>,
}

impl MpscQueue {
    pub fn new(size: usize) -> MpscQueue {
        let slots = if size & (size - 1) != 0 {
            round_to_power_of_2(size)
        } else {
            size
        };
        MpscQueue {
            slots: slots,
            mask: slots - 1,
            queue: (0..slots).map(|_| Default::default()).collect(),
            producer: Default::default(),
            consumer: Default::default(),
        }
    }

    #[inline]
    fn enqueue_mbufs(&self, start: usize, enqueue: usize, mbufs: &[*mut MBuf]) {
        let mask = self.mask;
        let len = self.slots;
        let mut mbuf_idx = 0;
        let mut queue_idx = start & mask;
        // FIXME: Unroll?
        if queue_idx + enqueue >= len {
            while queue_idx < len {
                self.queue[queue_idx].store(mbufs[mbuf_idx], Ordering::Release);
                mbuf_idx += 1;
                queue_idx += 1;
            }
            queue_idx = 0;
        }
        while mbuf_idx < enqueue {
            self.queue[queue_idx].store(mbufs[mbuf_idx], Ordering::Release);
            mbuf_idx += 1;
            queue_idx += 1;
        }
    }

    #[inline]
    pub fn enqueue(&self, mbufs: &[*mut MBuf]) -> usize {
        let len = mbufs.len();
        let mut insert;
        let mut producer_head;
        let mut consumer_tail;
        // First try and reserve memory by incrementing producer head.
        while {
            producer_head = self.producer.head.load(Ordering::Acquire);
            consumer_tail = self.consumer.tail.load(Ordering::Acquire);
            let free = self.mask.wrapping_add(consumer_tail).wrapping_sub(producer_head);
            insert = min(free, len);
            if insert == 0 {
                // Short circuit, no insertion
                false // This is the same as break in this construct.
            } else {
                let producer_next = producer_head.wrapping_add(insert);
                self.producer
                    .head
                    .compare_exchange(producer_head,
                                      producer_next,
                                      Ordering::AcqRel,
                                      Ordering::Relaxed)
                    .is_err()
            }
        } {}

        if insert > 0 {
            // If we successfully reserved memory, write to memory.
            let end = producer_head.wrapping_add(insert);
            self.enqueue_mbufs(producer_head, insert, &mbufs[..insert]);
            // Commit write by changing tail.
            // Before committing we wait for any preceding writes to finish (this is important since we assume buffer is
            // always available upto commit point.
            while {
                let producer_tail = self.producer.tail.load(Ordering::Acquire);
                producer_tail != producer_head
            } {
                pause(); // Pausing is a nice thing to do during spin locks
            }
            // Once this has been achieved, update tail. Any conflicting updates will wait on the previous spin lock.
            self.producer.tail.store(end, Ordering::Release);
            insert
        } else {
            0
        }

    }

    #[inline]
    pub fn enqueue_one(&self, mbuf: *mut MBuf) -> bool {
        self.enqueue(&[mbuf]) == 1
    }

    #[inline]
    fn dequeue_mbufs(&self, start: usize, dequeue: usize, mbufs: &mut [*mut MBuf]) {
        let mask = self.mask;
        let len = self.slots;
        // FIXME: Unroll?
        let mut mbuf_idx = 0;
        let mut queue_idx = start & mask;
        if queue_idx + dequeue >= len {
            while queue_idx < len {
                mbufs[mbuf_idx] = self.queue[queue_idx].load(Ordering::Acquire);
                mbuf_idx += 1;
                queue_idx += 1;
            }
            queue_idx = 0;
        }
        while mbuf_idx < dequeue {
            mbufs[mbuf_idx] = self.queue[queue_idx].load(Ordering::Acquire);
            mbuf_idx += 1;
            queue_idx += 1;
        }
    }

    #[inline]
    pub fn dequeue(&self, mbufs: &mut [*mut MBuf]) -> usize {
        // NOTE: This is a single consumer dequeue as assumed by this queue.
        let consumer_head = self.consumer.head.load(Ordering::Acquire);
        let producer_tail = self.producer.tail.load(Ordering::Acquire);
        let available_entries = producer_tail.wrapping_sub(consumer_head);
        let dequeue = min(mbufs.len(), available_entries);
        if dequeue > 0 {
            let consumer_next = consumer_head.wrapping_add(dequeue);
            // Reserve what we are going to dequeue.
            self.consumer.head.store(consumer_next, Ordering::Release);
            self.dequeue_mbufs(consumer_head, dequeue, mbufs);
            // Commit that we have dequeued.
            self.consumer.tail.store(consumer_next, Ordering::Release);
        }
        dequeue
    }
}

#[derive(Clone)]
pub struct MpscProducer {
    mpsc_queue: Arc<MpscQueue>,
}

impl MpscProducer {
    pub fn enqueue<T: EndOffset>(&self, packets: &mut [Packet<T>]) -> usize {
        let mbufs: Vec<_> = packets.iter_mut().map(|p| unsafe { p.get_mbuf() }).collect();
        self.mpsc_queue.enqueue(&mbufs[..])
    }

    pub fn enqueue_one<T: EndOffset>(&self, packet: &mut Packet<T>) -> bool {
        unsafe { self.mpsc_queue.enqueue_one(packet.get_mbuf()) }
    }
}

pub struct MpscConsumer {
    mpsc_queue: Arc<MpscQueue>,
}

impl ReceivableQueue for MpscConsumer {
    #[inline]
    fn receive_batch(&self, mbufs: &mut [*mut MBuf]) -> usize {
        self.mpsc_queue.dequeue(mbufs)
    }
}

pub fn new_mpsc_queue_pair_with_size(size: usize) -> (MpscProducer, ReceiveQueueGen<MpscConsumer>) {
    let mpsc_q = Arc::new(MpscQueue::new(size));
    (MpscProducer { mpsc_queue: mpsc_q.clone() }, ReceiveQueueGen::new(MpscConsumer {mpsc_queue: mpsc_q}))
}

const DEFAULT_QUEUE_SIZE: usize = 1024;

pub fn new_mpsc_queue_pair() -> (MpscProducer, ReceiveQueueGen<MpscConsumer>) {
    new_mpsc_queue_pair_with_size(DEFAULT_QUEUE_SIZE)
}
