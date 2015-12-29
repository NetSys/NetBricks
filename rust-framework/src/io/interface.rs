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
