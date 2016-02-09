pub trait ProcessPacketBatch {
    fn start(&self) -> usize;
    unsafe fn next_address(&mut self, idx: usize) -> Option<(*mut u8, usize)>;
    unsafe fn next_payload(&mut self, idx: usize) -> Option<(*mut u8, usize)>;
    unsafe fn payload(&mut self, idx: usize) -> *mut u8;
    unsafe fn address(&mut self, idx: usize) -> *mut u8;
}

pub struct PacketBatchAddressIterator<'a> {
    batch: &'a mut ProcessPacketBatch,
    idx: usize
}

impl<'a> PacketBatchAddressIterator<'a> {
    #[inline]
    pub fn new(batch: &mut ProcessPacketBatch) -> PacketBatchAddressIterator {
        let start = batch.start();
        PacketBatchAddressIterator{batch: batch, idx: start}
    }
}

impl<'a> Iterator for PacketBatchAddressIterator<'a> {
    type Item = *mut u8;

    #[inline]
    fn next(&mut self) -> Option<*mut u8> {
        let item = unsafe { self.batch.next_address(self.idx) };
        match item {
            Some((packet, idx)) => {
                self.idx = idx;
                Some(packet)
            },
            None => None
        }
    }
}
