use std::result;
mod dpdk {
    #[link(name = "zcsi")]
    extern "C" {
        pub fn init_system(name: *const u8, nlen: i32, core: i32) -> i32;
        pub fn init_system_whitelisted(name: *const u8,
                                       nlen: i32,
                                       core: i32,
                                       whitelist: *mut *const u8,
                                       wlcount: i32,
                                       pool_size: u32,
                                       cache_size: u32)
                                       -> i32;
        pub fn init_thread(tid: i32, core: i32);
        pub fn init_secondary(name: *const u8, nlen: i32, core: i32, vdevs: *mut *const u8, vdev_count: i32) -> i32;
    }
}

/// Initialize system. This must be run before any of the rest of this library is used.
/// Calling this function is somewhat slow.
/// # Failures: If a call to this function fails, DPDK will panic and kill the entire application.
pub fn init_system(name: &str, core: i32) {
    unsafe {
        let ret = dpdk::init_system(name.as_ptr(), name.len() as i32, core);
        if ret != 0 {
            panic!("Could not initialize the system errno {}", ret)
        }
    }
}

/// Initialize the system, whitelisting some set of NICs and allocating mempool of given size.
pub fn init_system_wl_with_mempool(name: &str, core: i32, pci: &[String], pool_size: u32, cache_size: u32) {
    let mut whitelist = Vec::<*const u8>::with_capacity(pci.len());
    for dev in pci {
        whitelist.push(dev.as_ptr());
    }
    unsafe {
        let ret = dpdk::init_system_whitelisted(name.as_ptr(),
                                                name.len() as i32,
                                                core,
                                                whitelist.as_mut_ptr(),
                                                pci.len() as i32,
                                                pool_size,
                                                cache_size);
        if ret != 0 {
            panic!("Could not initialize the system errno {}", ret)
        }
    }
}

const DEFAULT_POOL_SIZE: u32 = 2048 - 1;
const DEFAULT_CACHE_SIZE: u32 = 32;

/// Initialize the system, whitelisting some set of NICs.
pub fn init_system_wl(name: &str, core: i32, pci: &[String]) {
    init_system_wl_with_mempool(name, core, pci, DEFAULT_POOL_SIZE, DEFAULT_CACHE_SIZE);
}

/// Initialize the system as a DPDK secondary process with a set of VDEVs. User must specify mempool name to use.
pub fn init_system_secondary(name: &str, core: i32, vdevs: &[String]) {
    let mut vdev_list = Vec::<*const u8>::with_capacity(vdevs.len());
    for dev in vdevs {
        vdev_list.push(dev.as_ptr());
    }
    unsafe {
        let ret = dpdk::init_secondary(name.as_ptr(),
                                       name.len() as i32,
                                       core,
                                       vdev_list.as_mut_ptr(),
                                       vdevs.len() as i32);
        if ret != 0 {
            panic!("Could not initialize secondary process errno {}", ret)
        }
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
    CannotSend,
    BadVdev,
    BadTxQueue,
    BadRxQueue,
}

pub type Result<T> = result::Result<T, ZCSIError>;
