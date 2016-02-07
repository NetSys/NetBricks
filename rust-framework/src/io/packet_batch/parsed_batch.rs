use std::marker::PhantomData;
use super::Act;
use super::internal_iface::ProcessPacketBatch;
use super::packet_batch::cast_from_u8;
use super::TransformBatch;
use super::ApplyBatch;
use super::super::interface::EndOffset;

pub struct ParsedBatch<'a, T:'a + EndOffset, V> where
    V:'a + ProcessPacketBatch + Act {
    parent: &'a mut V,
    applied: bool,
    phantom: PhantomData<&'a T>,
}

impl<'a, T, V> Act for ParsedBatch<'a, T, V>
    where T:'a + EndOffset,
    V: 'a +  ProcessPacketBatch + Act {
    fn act(&mut self) -> &mut Self {
        self.parent.act();
        self
    }
}

batch!{ParsedBatch, [parent: &'a mut V], [phantom: PhantomData]}

impl<'a, T, V> ProcessPacketBatch for ParsedBatch<'a, T, V>
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
        let address = self.parent.payload(idx);
        let offset = T::offset(cast_from_u8::<T>(address));
        address.offset(offset as isize)
    }

    #[inline]
    unsafe fn address(&mut self, idx: usize) -> *mut u8 {
        self.parent.payload(idx)
    }
}
