use super::super::pmd::*;
use super::super::interface::Result;
pub trait Act {
    fn act(&mut self) -> &mut Self;

    /// Notification indicating we are done processing the current batch of packets
    fn done(&mut self) -> &mut Self;

    fn send_queue(&mut self, port: &mut PmdPort, queue: i32) -> Result<u32>;

    fn capacity(&self) -> i32;
}
