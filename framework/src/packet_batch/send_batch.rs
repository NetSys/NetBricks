extern crate time;
use io::PortQueue;
use io::Result;
use super::act::Act;
use super::Batch;
use super::iterator::*;
use std::any::Any;
use scheduler::Executable;

// FIXME: Should we be handling multiple queues and ports here?
// FIXME: Should this really even be a batch?
pub struct SendBatch<V>
    where V: Batch + BatchIterator + Act
{
    port: PortQueue,
    parent: V,
    pub sent: u64,
    batch: u64,
}

impl<V> SendBatch<V>
    where V: Batch + BatchIterator + Act
{
    pub fn new(parent: V, port: PortQueue) -> SendBatch<V> {
        SendBatch {
            port: port,
            sent: 0,
            parent: parent,
            batch: 0,
        }
    }
}

impl<V> Batch for SendBatch<V> where V: Batch + BatchIterator + Act {}

impl<V> BatchIterator for SendBatch<V>
    where V: Batch + BatchIterator + Act
{
    #[inline]
    fn start(&mut self) -> usize {
        panic!("Cannot iterate SendBatch")
    }

    #[inline]
    unsafe fn next_payload(&mut self, _: usize) -> Option<(PacketDescriptor, Option<&mut Any>, usize)> {
        panic!("Cannot iterate SendBatch")
    }

    #[inline]
    unsafe fn next_base_payload(&mut self, _: usize) -> Option<(PacketDescriptor, Option<&mut Any>, usize)> {
        panic!("Cannot iterate SendBatch")
    }

    #[inline]
    unsafe fn next_payload_popped(&mut self, _: usize, _: i32) -> Option<(PacketDescriptor, Option<&mut Any>, usize)> {
        panic!("Cannot iterate SendBatch")
    }
}

/// Internal interface for packets.
impl<V> Act for SendBatch<V>
    where V: Batch + BatchIterator + Act
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
        // First everything is applied
        self.parent.act();
        self.parent
            .send_q(&mut self.port)
            .and_then(|x| {
                self.sent += x as u64;
                if x > 0 {
                    self.batch += x as u64;
                    if self.batch > 3_200_000 {
                        let time = time::precise_time_ns();
                        println!("tx {} {} {}", self.port, self.batch, time);
                        self.batch = 0;
                    }
                }
                Ok(x)
            })
            .expect("Send failed");
        self.parent.done();
    }

    fn done(&mut self) {}

    fn send_q(&mut self, _: &mut PortQueue) -> Result<u32> {
        panic!("Cannot send a sent packet batch")
    }

    fn capacity(&self) -> i32 {
        self.parent.capacity()
    }

    #[inline]
    fn drop_packets(&mut self, _: Vec<usize>) -> Option<usize> {
        panic!("Cannot drop packets from a sent batch")
    }

    #[inline]
    fn adjust_payload_size(&mut self, _: usize, _: isize) -> Option<isize> {
        panic!("Cannot resize a sent batch")
    }

    #[inline]
    fn adjust_headroom(&mut self, _: usize, _: isize) -> Option<isize> {
        panic!("Cannot resize a sent batch")
    }
}

impl<V> Executable for SendBatch<V>
    where V: Batch + BatchIterator + Act
{
    #[inline]
    fn execute(&mut self) {
        self.act()
    }
}
