use io::PortQueue;
use io::Result;
use super::Batch;
pub trait Act {
    fn parent(&mut self) -> &mut Batch;

    fn parent_immutable(&self) -> &Batch;

    /// Actually perform whatever needs to be done by this processing node.
    fn act(&mut self) {
        self.parent().act();
    }

    /// Notification indicating we are done processing the current batch of packets
    fn done(&mut self) {
        self.parent().done();
    }

    fn send_q(&mut self, port: &mut PortQueue) -> Result<u32> {
        self.parent().send_q(port)
    }

    fn capacity(&self) -> i32 {
        self.parent_immutable().capacity()
    }

    fn drop_packets(&mut self, idxes: Vec<usize>) -> Option<usize> {
        self.parent().drop_packets(idxes)
    }

    /// Add bytes at the end of the packet. `size` is the new size requested, returns the new size after adjustment or 0
    /// if not done. Note `size` here is the amount by which packet size should change overall.
    fn adjust_payload_size(&mut self, idx: usize, size: isize) -> Option<isize> {
        self.parent().adjust_payload_size(idx, size)
    }

    fn adjust_headroom(&mut self, idx: usize, size: isize) -> Option<isize> {
        self.parent().adjust_headroom(idx, size)
    }
}
