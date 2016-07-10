use io::*;
pub trait ReceivableQueue {
    fn receive_batch(&self, &mut [*mut MBuf]) -> usize;
}
