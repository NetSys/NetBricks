pub use self::flow::*;
pub use self::spsc_mbuf_queue::*;
pub use self::asm::*;
pub use self::mpsc_mbuf_queue::*;
pub use self::receivable_queues::*;
mod flow;
mod spsc_mbuf_queue;
mod mpsc_mbuf_queue;
mod asm;
mod receivable_queues;

pub const PAGE_SIZE: usize = 4096; // Page size in bytes, not using huge pages here.

/// Round a given buffer to page size units.
#[inline]
pub fn round_to_pages(buffer_size: usize) -> usize {
    (buffer_size + (PAGE_SIZE - 1)) & !(PAGE_SIZE - 1)
}

/// Round a 64-bit integer to its nearest power of 2.
#[inline]
pub fn round_to_power_of_2(mut size: usize) -> usize {
    size = size.wrapping_sub(1);
    size |= size >> 1;
    size |= size >> 2;
    size |= size >> 4;
    size |= size >> 8;
    size |= size >> 16;
    size |= size >> 32;
    size = size.wrapping_add(1);
    size
}
