use io::PortQueue;
use io::Result;
use super::Batch;
use utils::SpscProducer;
pub trait Act {
    fn parent(&mut self) -> &mut Batch;

    fn parent_immutable(&self) -> &Batch;

    /// Actually perform whatever needs to be done by this processing node.
    #[inline]
    fn act(&mut self) {
        self.parent().act();
    }

    /// Notification indicating we are done processing the current batch of packets
    #[inline]
    fn done(&mut self) {
        self.parent().done();
    }

    #[inline]
    fn send_q(&mut self, port: &mut PortQueue) -> Result<u32> {
        self.parent().send_q(port)
    }

    #[inline]
    fn capacity(&self) -> i32 {
        self.parent_immutable().capacity()
    }

    #[inline]
    fn drop_packets(&mut self, idxes: Vec<usize>) -> Option<usize> {
        self.parent().drop_packets(idxes)
    }

    /// Add bytes at the end of the packet. `size` is the new size requested, returns the new size after adjustment or 0
    /// if not done. Note `size` here is the amount by which packet size should change overall.
    #[inline]
    fn adjust_payload_size(&mut self, idx: usize, size: isize) -> Option<isize> {
        self.parent().adjust_payload_size(idx, size)
    }

    #[inline]
    fn adjust_headroom(&mut self, idx: usize, size: isize) -> Option<isize> {
        self.parent().adjust_headroom(idx, size)
    }

    #[inline]
    fn distribute_to_queues(&mut self,
                            queues: &[SpscProducer],
                            groups: Vec<(usize, usize)>,
                            free_if_not_enqueued: bool) {
        self.parent().distribute_to_queues(queues, groups, free_if_not_enqueued)
    }
}
