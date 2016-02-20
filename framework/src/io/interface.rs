use std::result;
mod dpdk {
    #[link(name = "zcsi")]
    extern {
        pub fn init_system(core: i32);
        pub fn init_thread(tid: i32, core: i32);
        pub fn init_system_whitelisted(core: i32, whitelist: *mut *const u8, wlcount: i32);
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

pub fn init_system_wl(core: i32, pci: &Vec<String>) {
    let mut whitelist = Vec::<*const u8>::with_capacity(pci.len());
    for l in 0..pci.len() {
        let dev = &pci[l];
        whitelist.push(dev.as_ptr());
    }
    unsafe {
        dpdk::init_system_whitelisted(core, whitelist.as_mut_ptr(), pci.len() as i32);
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
