use common::*;
use headers::NullHeader;
use interface::PacketTx;
use scheduler::Executable;
use super::Batch;
use super::act::Act;
use super::iterator::*;
use super::packet_batch::PacketBatch;

pub struct SendBatch<Port, V>
    where Port: PacketTx,
          V: Batch + BatchIterator + Act
{
    port: Port,
    parent: V,
    pub sent: u64,
}

impl<Port, V> SendBatch<Port, V>
    where Port: PacketTx,
          V: Batch + BatchIterator + Act
{
    pub fn new(parent: V, port: Port) -> SendBatch<Port, V> {
        SendBatch {
            port: port,
            sent: 0,
            parent: parent,
        }
    }
}

impl<Port, V> Batch for SendBatch<Port, V> where Port: PacketTx, V: Batch + BatchIterator + Act {}

impl<Port, V> BatchIterator for SendBatch<Port, V>
    where Port: PacketTx,
          V: Batch + BatchIterator + Act
{
    type Header = NullHeader;
    type Metadata = EmptyMetadata;
    #[inline]
    fn start(&mut self) -> usize {
        panic!("Cannot iterate send batch")
    }

    #[inline]
    unsafe fn next_payload(&mut self, _: usize) -> Option<PacketDescriptor<NullHeader, EmptyMetadata>> {
        panic!("Cannot iterate send batch")
    }
}

/// Internal interface for packets.
impl<Port, V> Act for SendBatch<Port, V>
    where Port: PacketTx,
          V: Batch + BatchIterator + Act
{
    #[inline]
    fn act(&mut self) {
        // First everything is applied
        self.parent.act();
        self.parent
            .get_packet_batch()
            .send_q(&self.port)
            .and_then(|x| {
                self.sent += x as u64;
                Ok(x)
            })
            .expect("Send failed");
        self.parent.done();
    }

    fn done(&mut self) {}

    fn send_q(&mut self, _: &PacketTx) -> Result<u32> {
        panic!("Cannot send a sent packet batch")
    }

    fn capacity(&self) -> i32 {
        self.parent.capacity()
    }

    #[inline]
    fn drop_packets(&mut self, _: &[usize]) -> Option<usize> {
        panic!("Cannot drop packets from a sent batch")
    }

    #[inline]
    fn clear_packets(&mut self) {
        panic!("Cannot clear packets from a sent batch")
    }

    #[inline]
    fn get_packet_batch(&mut self) -> &mut PacketBatch {
        self.parent.get_packet_batch()
    }
}

impl<Port, V> Executable for SendBatch<Port, V>
    where Port: PacketTx,
          V: Batch + BatchIterator + Act
{
    #[inline]
    fn execute(&mut self) {
        self.act()
    }
}
