use std::result;
extern crate libc;

mod dpdk {
    #[link(name = "zcsi")]
    extern {
        pub fn init_system(core: i32);
        pub fn init_thread(tid: i32, core: i32);
    }
}

/// Initialize system. This must be run before any of the rest of this library is used.
/// Calling this function is somewhat slow.
/// # Failures: If a call to this function fails, DPDK will panic and kill the entire application.
pub fn init_system(core: i32) {
    unsafe {
        dpdk::init_system(core);
    }
}

/// Affinitize a pthread to a core and assign a DPDK thread ID.
pub fn init_thread(tid: i32, core: i32) {
    unsafe {
        dpdk::init_thread(tid, core);
    }
}

#[derive(Debug)]
pub enum ZCSIError {
    FailedAllocation,
    FailedDeallocation,
    FailedToInitializePort,
    BadQueue,
}

pub type Result<T> = result::Result<T, ZCSIError>;

pub trait EndOffset {
    /// Offset returns the number of bytes to skip to get to the next header.
    fn offset(&self) -> usize; 
    /// Returns the size of this header in bytes.
    fn size() -> usize;
}
