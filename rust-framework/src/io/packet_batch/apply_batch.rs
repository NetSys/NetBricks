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

impl<'a, T, V> ApplyBatch<'a, T, V>
    where T: 'a + EndOffset,
    V:'a + ProcessPacketBatch + Act {

    pub fn new(parent: &'a mut V, template: &'a T) -> ApplyBatch<'a, T, V> {
        ApplyBatch{ parent: parent, template: template, applied:false, }
    }
    
    // FIXME: Rename this to something reasonable
    //#[inline]
    pub fn parse<T2: EndOffset>(&mut self) -> ParsedBatch<T2, ApplyBatch<'a, T, V>> {
        ParsedBatch::<T2, Self>::new(self)
    }

    //#[inline]
    pub fn transform(&'a mut self, transformer: &'a Fn(&mut T)) -> TransformBatch<T, Self> {
        TransformBatch::<T, Self>::new(self, transformer)
    }

    // FIXME: Turn this into a node, rather than having it act this way
    //#[inline]
    pub fn deparse(&'a mut self) -> &'a mut V {
        if !self.applied {
            self.act();
        }
        self.parent
    }
}

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
