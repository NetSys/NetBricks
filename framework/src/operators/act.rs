use super::packet_batch::PacketBatch;
use common::*;
use interface::PacketTx;
pub trait Act {
    /// Actually perform whatever needs to be done by this processing node.
    #[inline]
    fn act(&mut self);

    /// Notification indicating we are done processing the current batch of packets
    #[inline]
    fn done(&mut self);

    #[inline]
    fn send_q(&mut self, port: &PacketTx) -> Result<u32>;

    #[inline]
    fn capacity(&self) -> i32;

    #[inline]
    fn drop_packets(&mut self, idxes: &[usize]) -> Option<usize>;

    /// Remove all packets from the batch (without actually freeing them).
    #[inline]
    fn clear_packets(&mut self) {
        self.get_packet_batch().clear_packets();
    }

    #[inline]
    fn get_packet_batch(&mut self) -> &mut PacketBatch;

    /// Get tasks that feed produce packets for this batch. We use this in the embedded scheduler.
    #[inline]
    fn get_task_dependencies(&self) -> Vec<usize>;
}
