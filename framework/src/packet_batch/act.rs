use io::PmdPort;
use io::Result;
pub trait Act {
    /// Actually perform whatever needs to be done by this processing node.
    fn act(&mut self);

    /// Notification indicating we are done processing the current batch of packets
    fn done(&mut self);

    fn send_queue(&mut self, port: &mut PmdPort, queue: i32) -> Result<u32>;

    fn capacity(&self) -> i32;

    fn drop_packets(&mut self, idxes: Vec<usize>) -> Option<usize>;

    /// Add bytes at the end of the packet. `size` is the new size requested, returns the new size after adjustment or 0
    /// if not done. Note `size` here is the amount by which packet size should change overall.
    fn adjust_payload_size(&mut self, idx: usize, size: isize) -> Option<isize>;
}
