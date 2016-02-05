use std::marker::PhantomData;
use super::Act;
use super::internal_iface::ProcessPacketBatch;
use super::packet_batch::cast_from_u8;
use super::TransformBatch;
use super::super::interface::EndOffset;

pub struct ParsedBatch<'a, T:'a + EndOffset, V> where
    V:'a + ProcessPacketBatch + Act {
    parent: &'a mut V,
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

impl<'a, T, V> ParsedBatch<'a, T, V>
    where T:'a + EndOffset,
    V: 'a +  ProcessPacketBatch + Act {

    pub fn new(parent: &'a mut V) -> ParsedBatch<'a, T, V> {
        ParsedBatch{parent: parent, phantom: PhantomData}
    }

    // FIXME: Rename this to something reasonable
    #[inline]
    pub fn parse<T2: EndOffset>(&mut self) -> ParsedBatch<T2, Self> {
        ParsedBatch::<T2, Self>{ parent: self, phantom: PhantomData }
    }

    #[inline]
    pub fn transform(&'a mut self, transformer: &'a Fn(&mut T)) -> TransformBatch<T, Self> {
        TransformBatch::<T, Self>::new(self, transformer)
    }

    #[inline]
    pub fn deparse(&'a mut self) -> &'a mut V {
        self.parent
    }
}

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
