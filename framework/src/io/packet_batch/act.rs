use super::super::pmd::*;
use super::super::interface::Result;
pub trait Act {
    /// Actually perform whatever needs to be done by this processing node.
    fn act(&mut self);

    /// Notification indicating we are done processing the current batch of packets
    fn done(&mut self);

    fn send_queue(&mut self, port: &mut PmdPort, queue: i32) -> Result<u32>;

    fn capacity(&self) -> i32;

    fn drop_packets(&mut self, idxes: Vec<usize>) -> Option<usize>;
}
