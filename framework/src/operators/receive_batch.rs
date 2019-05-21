use super::{Batch, PacketError};
use interface::PacketRx;
use native::mbuf::MBuf;
use packets::RawPacket;

pub const BATCH_SIZE: usize = 32;

/// Receive operator
///
/// Marks the start of a pipeline.
pub struct ReceiveBatch<Rx: PacketRx> {
    port: Rx,
    buffers: Vec<*mut MBuf>,
    index: usize,
}

impl<Rx: PacketRx> ReceiveBatch<Rx> {
    #[inline]
    pub fn new(port: Rx) -> Self {
        ReceiveBatch {
            port,
            buffers: Vec::<*mut MBuf>::with_capacity(BATCH_SIZE),
            index: 0,
        }
    }
}

impl<Rx: PacketRx> Batch for ReceiveBatch<Rx> {
    type Item = RawPacket;

    #[inline]
    fn next(&mut self) -> Option<Result<Self::Item, PacketError>> {
        // TODO: better if this is a queue
        if self.buffers.len() > self.index {
            let mbuf = self.buffers[self.index];
            self.index += 1;
            Some(Ok(RawPacket::from_mbuf(mbuf)))
        } else {
            self.buffers.clear();
            self.index = 0;
            None
        }
    }

    #[inline]
    fn receive(&mut self) {
        unsafe {
            let capacity = self.buffers.capacity();
            self.buffers.set_len(capacity);
            match self.port.recv(self.buffers.as_mut_slice()) {
                Ok(received) => self.buffers.set_len(received as usize),
                // the underlying DPDK method `rte_eth_rx_burst` will
                // never return an error. The error arm is unreachable
                _ => unreachable!(),
            }
        }
    }
}
