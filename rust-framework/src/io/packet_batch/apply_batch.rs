use super::internal_iface::ProcessPacketBatch;
use super::TransformBatch;
use super::ParsedBatch;
use super::Act;
use super::packet_batch::cast_from_u8;
use super::super::interface::EndOffset;
use std::ptr;

pub struct ApplyBatch<'a, T, V>
    where T: 'a + EndOffset,
    V:'a + ProcessPacketBatch + Act {
    parent: &'a mut V,
    template: &'a T,
    applied: bool,
}

batch!{ApplyBatch, [parent: &'a mut V, template: &'a T], []}

impl<'a, T, V> Act for ApplyBatch<'a, T, V>
    where T:'a + EndOffset,
    V: 'a +  ProcessPacketBatch + Act {
    fn act(&mut self) -> &mut Self {
        let start = self.start();
        let end = self.end();
        let mut idx = start;
        while idx < end {
            unsafe {
                let address = cast_from_u8::<T>(self.parent.address(idx));
                ptr::copy_nonoverlapping(self.template, address, 1);
            }
            idx += 1;
        }
        self.applied = true;
        self.parent.act();
        self
    }
}

impl<'a, T, V> ProcessPacketBatch for ApplyBatch<'a, T, V>
    where T:'a + EndOffset,
    V: 'a +  ProcessPacketBatch + Act {
    #[inline]
    fn start(&self) -> usize {
        self.parent.start()
    }

    #[inline]
    fn end(&self) -> usize {
        self.parent.end()
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
}
