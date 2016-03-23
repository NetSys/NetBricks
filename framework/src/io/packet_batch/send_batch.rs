use super::act::Act;
use super::Batch;
use super::iterator::BatchIterator;
use super::super::pmd::*;
use super::super::interface::Result;

// FIXME: Should we be handling multiple queues and ports here?
// FIXME: Should this really even be a batch?
pub struct SendBatch<'a, V>
    where V: 'a + Batch + BatchIterator + Act
{
    port: &'a mut PmdPort,
    queue: i32,
    parent: V,
    pub sent: u64,
}

impl<'a, V> SendBatch<'a, V>
    where V: 'a + Batch + BatchIterator + Act
{
    pub fn new(parent: V, port: &'a mut PmdPort, queue: i32) -> SendBatch<'a, V> {
        SendBatch {
            port: port,
            queue: queue,
            sent: 0,
            parent: parent,
        }
    }

    pub fn process(&mut self) {
        self.act();
    }
}

impl<'a, V> Batch for SendBatch<'a, V>
    where V: 'a + Batch + BatchIterator + Act
{
    type Parent = V;

    fn pop(&mut self) -> &mut V {
        panic!("Cannot get parent of sent batch")
    }
}

// FIXME: All these should panic instead of doing this.
impl<'a, V> BatchIterator for SendBatch<'a, V>
    where V: 'a + Batch + BatchIterator + Act
{
    #[inline]
    fn start(&mut self) -> usize {
        panic!("Cannot iterate SendBatch")
    }

    #[inline]
    unsafe fn next_address(&mut self, _: usize) -> Option<(*mut u8, usize)> {
        panic!("Cannot iterate SendBatch")
    }

    #[inline]
    unsafe fn next_payload(&mut self, _: usize) -> Option<(*mut u8, usize)> {
        panic!("Cannot iterate SendBatch")
    }

    #[inline]
    unsafe fn next_base_address(&mut self, _: usize) -> Option<(*mut u8, usize)> {
        panic!("Cannot iterate SendBatch")
    }

    #[inline]
    unsafe fn next_base_payload(&mut self, _: usize) -> Option<(*mut u8, usize)> {
        panic!("Cannot iterate SendBatch")
    }
}

/// Internal interface for packets.
impl<'a, V> Act for SendBatch<'a, V>
    where V: 'a + Batch + BatchIterator + Act
{
    #[inline]
    fn act(&mut self) -> &mut Self {
        // First everything is applied
        self.parent.act();
        self.parent
            .send_queue(self.port, self.queue)
            .and_then(|x| {
                self.sent += x as u64;
                Ok(x)
            })
            .expect("Send failed");
        self.parent.done();
        self
    }

    fn done(&mut self) -> &mut Self {
        self
    }

    fn send_queue(&mut self, port: &mut PmdPort, queue: i32) -> Result<u32> {
        self.parent.send_queue(port, queue)
    }

    fn capacity(&self) -> i32 {
        self.parent.capacity()
    }

    #[inline]
    fn drop_packets(&mut self, idxes: Vec<usize>) -> Option<usize> {
        self.parent.drop_packets(idxes)
    }
}
