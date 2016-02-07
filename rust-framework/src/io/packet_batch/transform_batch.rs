use super::internal_iface::ProcessPacketBatch;
use super::ParsedBatch;
use super::ApplyBatch;
use super::Act;
use super::packet_batch::cast_from_u8;
use super::super::interface::EndOffset;

pub struct TransformBatch<'a, T, V>
    where T: 'a + EndOffset,
    V:'a + ProcessPacketBatch + Act {
    parent: &'a mut V,
    transformer: &'a Fn(&'a mut T),
    applied: bool,
}

batch! {TransformBatch, [parent : &'a mut V, transformer: &'a Fn(&'a mut T)], []}

impl<'a, T, V> Act for TransformBatch<'a, T, V>
    where T:'a + EndOffset,
    V: 'a +  ProcessPacketBatch + Act {
    fn act(&mut self) -> &mut Self {
        let start = self.start();
        let end = self.end();
        let f = self.transformer;
        let mut idx = start;
        while idx < end {
            let address = unsafe {self.parent.address(idx)};
            f(cast_from_u8::<T>(address));
            idx += 1;
        }
        self.applied = true;
        self.parent.act();
        self
    }
}

impl<'a, T, V> ProcessPacketBatch for TransformBatch<'a, T, V>
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
