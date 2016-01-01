pub trait ProcessPacketBatch : Sized {
    fn start(&self) -> usize;
    fn end(&self) -> usize;
    unsafe fn payload(&mut self, idx: usize) -> *mut u8;
    unsafe fn address(&mut self, idx: usize) -> *mut u8;
}
