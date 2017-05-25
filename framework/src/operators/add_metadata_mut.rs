use super::Batch;
use super::act::Act;
use super::iterator::*;
use super::packet_batch::PacketBatch;
use common::*;
use interface::Packet;
use interface::PacketTx;
use std::marker::PhantomData;

pub type MutableMetadataFn<T, M, M2> = Box<FnMut(&mut Packet<T, M>) -> M2 + Send>;

pub struct MutableAddMetadataBatch<M, V>
    where M: Send + Sized,
          V: Batch + BatchIterator + Act
{
    parent: V,
    generator: MutableMetadataFn<V::Header, V::Metadata, M>,
    applied: bool,
    _phantom_m: PhantomData<M>,
}

impl<M, V> MutableAddMetadataBatch<M, V>
    where M: Send + Sized,
          V: Batch + BatchIterator + Act
{
    pub fn new(parent: V, generator: MutableMetadataFn<V::Header, V::Metadata, M>) -> MutableAddMetadataBatch<M, V> {
        MutableAddMetadataBatch {
            parent: parent,
            generator: generator,
            applied: false,
            _phantom_m: PhantomData,
        }
    }
}

impl<M, V> Batch for MutableAddMetadataBatch<M, V>
    where M: Send + Sized,
          V: Batch + BatchIterator + Act
{
}

impl<M, V> BatchIterator for MutableAddMetadataBatch<M, V>
    where M: Send + Sized,
          V: Batch + BatchIterator + Act
{
    type Header = V::Header;
    type Metadata = M;

    #[inline]
    fn start(&mut self) -> usize {
        self.parent.start()
    }

    #[inline]
    unsafe fn next_payload(&mut self, idx: usize) -> Option<PacketDescriptor<V::Header, M>> {
        self.parent
            .next_payload(idx)
            .map(|p| PacketDescriptor { packet: p.packet.reinterpret_metadata() })
    }
}

impl<M, V> Act for MutableAddMetadataBatch<M, V>
    where M: Send + Sized,
          V: Batch + BatchIterator + Act
{
    #[inline]
    fn act(&mut self) {
        if !self.applied {
            self.parent.act();
            {
                let iter = PayloadEnumerator::<V::Header, V::Metadata>::new(&mut self.parent);
                while let Some(ParsedDescriptor { mut packet, .. }) = iter.next(&mut self.parent) {
                    let metadata = (self.generator)(&mut packet);
                    packet.write_metadata(&metadata).unwrap(); // FIXME: WHat to do on error?
                }
            }
            self.applied = true;
        }
    }

    #[inline]
    fn done(&mut self) {
        self.applied = false;
        self.parent.done();
    }

    #[inline]
    fn send_q(&mut self, port: &PacketTx) -> Result<u32> {
        self.parent.send_q(port)
    }

    #[inline]
    fn capacity(&self) -> i32 {
        self.parent.capacity()
    }

    #[inline]
    fn drop_packets(&mut self, idxes: &[usize]) -> Option<usize> {
        self.parent.drop_packets(idxes)
    }

    #[inline]
    fn clear_packets(&mut self) {
        self.parent.clear_packets()
    }

    #[inline]
    fn get_packet_batch(&mut self) -> &mut PacketBatch {
        self.parent.get_packet_batch()
    }

    #[inline]
    fn get_task_dependencies(&self) -> Vec<usize> {
        self.parent.get_task_dependencies()
    }
}
