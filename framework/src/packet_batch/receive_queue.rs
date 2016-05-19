use io::PortQueue;
use io::Result;
use super::act::Act;
use super::Batch;
use super::packet_batch::PacketBatch;
use super::iterator::*;
use std::any::Any;
use utils::SpscConsumer;
use std::marker::PhantomData;

// FIXME: Should we be handling multiple queues and ports here?
pub struct ReceiveQueue<S>
    where S: 'static + Any + Default + Clone + Sized + Send
{
    parent: PacketBatch,
    meta: Vec<Box<S>>,
    queue: SpscConsumer<u8>,
    raw: Vec<*mut u8>,
    pub received: u64,
    phantom_s: PhantomData<S>,
}

// *mut u8 is not send by default.
unsafe impl<S> Send for ReceiveQueue<S> 
    where S: 'static + Any + Default + Clone + Sized + Send
{
}

impl<S> ReceiveQueue<S>
    where S: 'static + Any + Default + Clone + Sized + Send
{
    pub fn new_with_parent(parent: PacketBatch, queue: SpscConsumer<u8>) -> ReceiveQueue<S> {
        let capacity = parent.capacity() as usize;
        ReceiveQueue {
            parent: parent,
            meta: Vec::with_capacity(capacity),
            raw: Vec::with_capacity(capacity),
            queue: queue,
            received: 0,
            phantom_s: PhantomData,
        }
    }

    pub fn new(queue: SpscConsumer<u8>) -> ReceiveQueue<S> {
        ReceiveQueue {
            parent: PacketBatch::new(32),
            meta: Vec::with_capacity(32),
            raw: Vec::with_capacity(32),
            queue: queue,
            received: 0,
            phantom_s: PhantomData,
        }

    }
}

impl<S> Batch for ReceiveQueue<S> where S: 'static + Any + Default + Clone + Sized + Send {}

impl<S> BatchIterator for ReceiveQueue<S>
    where S: 'static + Any + Default + Clone + Sized + Send
{
    #[inline]
    fn start(&mut self) -> usize {
        self.parent.start()
    }

    #[inline]
    unsafe fn next_payload(&mut self, idx: usize) -> Option<(PacketDescriptor, Option<&mut Any>, usize)> {
        match self.parent.next_payload(idx) {
            Some((p, _, i)) => Some((p, self.meta.get_mut(idx).and_then(|x| Some(x as &mut Any)), i)),
            None => None,
        }
    }

    #[inline]
    unsafe fn next_base_payload(&mut self, idx: usize) -> Option<(PacketDescriptor, Option<&mut Any>, usize)> {
        match self.parent.next_base_payload(idx) {
            Some((p, _, i)) => Some((p, self.meta.get_mut(idx).and_then(|x| Some(x as &mut Any)), i)),
            None => None,
        }
    }

    #[inline]
    unsafe fn next_payload_popped(&mut self,
                                  idx: usize,
                                  pop: i32)
                                  -> Option<(PacketDescriptor, Option<&mut Any>, usize)> {
        match self.parent.next_payload_popped(idx, pop) { 
            Some((p, _, i)) => Some((p, self.meta.get_mut(idx).and_then(|x| Some(x as &mut Any)), i)),
            None => None,
        }
    }
}

/// Internal interface for packets.
impl<S> Act for ReceiveQueue<S>
    where S: 'static + Any + Default + Clone + Sized + Send
{
    #[inline]
    fn parent(&mut self) -> &mut Batch {
        &mut self.parent
    }

    #[inline]
    fn parent_immutable(&self) -> &Batch {
        &self.parent
    }

    #[inline]
    fn act(&mut self) {
        self.parent.act();
        self.parent
            .recv_spsc_queue(&self.queue, &mut self.raw)
            .and_then(|x| {
                self.received += x as u64;
                Ok(x)
            })
            .expect("Receive failed");
        self.meta.clear();
        for meta in self.raw.drain(..) {
            if meta.is_null() {
                self.meta.push(Default::default());
            } else {
                unsafe {
                    self.meta.push(Box::<S>::from_raw(meta as *mut S));
                }
            }
        }
    }

    #[inline]
    fn done(&mut self) {
        // Free up memory
        self.parent.deallocate_batch().expect("Deallocation failed");
        self.meta.clear();
    }

    #[inline]
    fn send_q(&mut self, port: &mut PortQueue) -> Result<u32> {
        self.parent.send_q(port)
    }

    #[inline]
    fn capacity(&self) -> i32 {
        self.parent.capacity()
    }

    #[inline]
    fn drop_packets(&mut self, idxes: &Vec<usize>) -> Option<usize> {
        self.parent.drop_packets(idxes)
    }

    #[inline]
    fn adjust_payload_size(&mut self, idx: usize, size: isize) -> Option<isize> {
        self.parent.adjust_payload_size(idx, size)
    }

    #[inline]
    fn adjust_headroom(&mut self, idx: usize, size: isize) -> Option<isize> {
        self.parent.adjust_headroom(idx, size)
    }
}
