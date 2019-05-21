use super::{Batch, PacketError, BATCH_SIZE};
use interface::PacketTx;
use native::mbuf::MBuf;
use native::zcsi::mbuf_free_bulk;
use packets::Packet;
use scheduler::Executable;

/// Send operator
///
/// Marks the end of a pipeline.
pub struct SendBatch<B: Batch, Tx: PacketTx> {
    source: B,
    port: Tx,
    transmit_q: Vec<*mut MBuf>,
    drop_q: Vec<*mut MBuf>,
}

impl<B: Batch, Tx: PacketTx> SendBatch<B, Tx> {
    #[inline]
    pub fn new(source: B, port: Tx) -> Self {
        SendBatch {
            source,
            port,
            transmit_q: Vec::with_capacity(BATCH_SIZE),
            drop_q: Vec::with_capacity(BATCH_SIZE),
        }
    }
}

impl<B: Batch, Tx: PacketTx> Executable for SendBatch<B, Tx> {
    fn execute(&mut self) {
        self.source.receive();

        let transmit_q = &mut self.transmit_q;
        let drop_q = &mut self.drop_q;

        while let Some(item) = self.source.next() {
            match item {
                Ok(packet) => {
                    transmit_q.push(packet.mbuf());
                }
                Err(PacketError::Emit(mbuf)) => {
                    transmit_q.push(mbuf);
                }
                Err(PacketError::Drop(mbuf)) => {
                    drop_q.push(mbuf);
                }
                Err(PacketError::Abort(mbuf, err)) => {
                    error_chain!(&err);
                    drop_q.push(mbuf);
                }
            }
        }

        if !transmit_q.is_empty() {
            let mut to_send = transmit_q.len();
            while to_send > 0 {
                match self.port.send(transmit_q.as_mut_slice()) {
                    Ok(sent) => {
                        let sent = sent as usize;
                        to_send -= sent;
                        if to_send > 0 {
                            transmit_q.drain(..sent);
                        }
                    }
                    // the underlying DPDK method `rte_eth_tx_burst` will
                    // never return an error. The error arm is unreachable
                    _ => unreachable!(),
                }
            }
            unsafe {
                transmit_q.set_len(0);
            }
        }

        if !drop_q.is_empty() {
            let len = drop_q.len();
            let ptr = drop_q.as_mut_ptr();
            unsafe {
                // never have a non-zero return
                mbuf_free_bulk(ptr, len as i32);
                drop_q.set_len(0);
            }
        }
    }

    #[inline]
    fn dependencies(&mut self) -> Vec<usize> {
        vec![]
    }
}
