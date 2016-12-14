pub use self::mpsc_mbuf_queue::*;

mod mpsc_mbuf_queue;

use native::zcsi::MBuf;
pub trait ReceivableQueue {
    fn receive_batch(&self, &mut [*mut MBuf]) -> usize;
}
