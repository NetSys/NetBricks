use super::internal_iface::{ProcessPacketBatch, PacketBatchAddressIterator};
use super::TransformBatch;
use super::ParsedBatch;
use super::Act;
use super::packet_batch::cast_from_u8;
use super::super::interface::EndOffset;
use std::ptr;

pub struct ReplaceBatch<'a, T, V>
    where T: 'a + EndOffset,
    V:'a + ProcessPacketBatch + Act {
    parent: &'a mut V,
    template: &'a T,
    applied: bool,
}

batch!{ReplaceBatch, [parent: &'a mut V, template: &'a T]}

impl<'a, T, V> Act for ReplaceBatch<'a, T, V>
    where T:'a + EndOffset,
    V: 'a +  ProcessPacketBatch + Act {
    fn act(&mut self) -> &mut Self {
        {
            let iter = PacketBatchAddressIterator::new(self.parent);
            for addr in iter {
                unsafe {
                    let address = cast_from_u8::<T>(addr);
                    ptr::copy_nonoverlapping(self.template, address, 1);
                }
            }
        }
        self.applied = true;
        self.parent.act();
        self
    }
}

impl<'a, T, V> ProcessPacketBatch for ReplaceBatch<'a, T, V>
    where T:'a + EndOffset,
    V: 'a +  ProcessPacketBatch + Act {
    #[inline]
    fn start(&self) -> usize {
        self.parent.start()
    }

    #[inline]
    unsafe fn payload(&mut self, idx: usize) -> *mut u8 {
        if !self.applied {
            self.act();
        }
        self.parent.payload(idx)
    }

    #[inline]
    unsafe fn address(&mut self, idx: usize) -> *mut u8 {
        if !self.applied {
            self.act();
        }
        self.parent.address(idx)
    }

    #[inline]
    unsafe fn next_address(&mut self, idx: usize) -> Option<(*mut u8, usize)> {
        if !self.applied {
            self.act();
        }
        self.parent.next_address(idx)
    }

    #[inline]
    unsafe fn next_payload(&mut self, idx: usize) -> Option<(*mut u8, usize)> {
        if !self.applied {
            self.act();
        }
        self.parent.next_payload(idx)
    }
}
