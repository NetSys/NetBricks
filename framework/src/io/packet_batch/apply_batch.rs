use super::iterator::{BatchIterator, PacketBatchAddressIterator};
use super::Act;
use super::Batch;
use super::super::pmd::*;
use super::packet_batch::cast_from_u8;
use super::super::interface::EndOffset;
use std::ptr;
use super::super::interface::Result;

pub struct ReplaceBatch<'a, T, V>
    where T: 'a + EndOffset,
          V: 'a + Batch + BatchIterator + Act
{
    parent: &'a mut V,
    template: &'a T,
    applied: bool,
}

batch!{ReplaceBatch, [parent: &'a mut V, template: &'a T]}

impl<'a, T, V> Act for ReplaceBatch<'a, T, V>
    where T: 'a + EndOffset,
          V: 'a + Batch + BatchIterator + Act
{
    fn act(&mut self) -> &mut Self {
        // This inner context is to allow the iter reference to expire before we change self.
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

    fn done(&mut self) -> &mut Self {
        self.applied = false;
        self.parent.done();
        self
    }

    fn send_queue(&mut self, port: &mut PmdPort, queue: i32) -> Result<u32> {
        self.parent.send_queue(port, queue)
    }
}

impl<'a, T, V> BatchIterator for ReplaceBatch<'a, T, V>
    where T: 'a + EndOffset,
          V: 'a + Batch + BatchIterator + Act
{
    //FIXME: We should just go from packet batch applying, instead of doing this version where act() is triggered as a
    //result of some functions being called.
    #[inline]
    fn start(&mut self) -> usize {
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
