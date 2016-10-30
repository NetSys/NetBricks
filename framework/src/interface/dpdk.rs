use std::ffi::CString;
use super::METADATA_SLOTS;
use config::{DEFAULT_CACHE_SIZE, DEFAULT_POOL_SIZE, NetbricksConfiguration};
mod native {
    use std::os::raw::c_char;
    #[link(name = "zcsi")]
    extern "C" {
        pub fn init_system_whitelisted(name: *const c_char,
                                       nlen: i32,
                                       core: i32,
                                       whitelist: *mut *const c_char,
                                       wlcount: i32,
                                       pool_size: u32,
                                       cache_size: u32,
                                       slots: u16)
                                       -> i32;
        pub fn init_thread(tid: i32, core: i32);
        pub fn init_secondary(name: *const c_char,
                              nlen: i32,
                              core: i32,
                              vdevs: *mut *const c_char,
                              vdev_count: i32)
                              -> i32;
    }
}

/// Initialize the system, whitelisting some set of NICs and allocating mempool of given size.
pub fn init_system_wl_with_mempool(name: &str, core: i32, pci: &[String], pool_size: u32, cache_size: u32) {
    let name_cstr = CString::new(name).unwrap();
    let pci_cstr: Vec<_> = pci.iter().map(|p| CString::new(&p[..]).unwrap()).collect();
    let mut whitelist: Vec<_> = pci_cstr.iter().map(|p| p.as_ptr()).collect();
    unsafe {
        let ret = native::init_system_whitelisted(name_cstr.as_ptr(),
                                                  name.len() as i32,
                                                  core,
                                                  whitelist.as_mut_ptr(),
                                                  pci.len() as i32,
                                                  pool_size,
                                                  cache_size,
                                                  METADATA_SLOTS);
        if ret != 0 {
            panic!("Could not initialize the system errno {}", ret)
        }
    }
}

/// Initialize the system, whitelisting some set of NICs.
pub fn init_system_wl(name: &str, core: i32, pci: &[String]) {
    init_system_wl_with_mempool(name, core, pci, DEFAULT_POOL_SIZE, DEFAULT_CACHE_SIZE);
}

/// Initialize the system as a DPDK secondary process with a set of VDEVs. User must specify mempool name to use.
pub fn init_system_secondary(name: &str, core: i32) {
    let name_cstr = CString::new(name).unwrap();
    let mut vdev_list = vec![];
    unsafe {
        let ret = native::init_secondary(name_cstr.as_ptr(),
                                         name.len() as i32,
                                         core,
                                         vdev_list.as_mut_ptr(),
                                         0);
        if ret != 0 {
            panic!("Could not initialize secondary process errno {}", ret)
        }
    }
}

/// Initialize the system based on the supplied scheduler configuration.
pub fn init_system(config: &NetbricksConfiguration) {
    if config.name.is_empty() {
        panic!("Configuration must provide a name.");
    }
    // We init with all devices blacklisted and rely on the attach logic to white list them as necessary.
    if config.secondary {
        // We do not have control over any of the other settings in this case.
        init_system_secondary(&config.name[..], config.primary_core);
    } else {
        init_system_wl_with_mempool(&config.name[..],
                                    config.primary_core,
                                    &[],
                                    config.pool_size,
                                    config.cache_size);
    }
}

/// Affinitize a pthread to a core and assign a DPDK thread ID.
pub fn init_thread(tid: i32, core: i32) {
    unsafe {
        native::init_thread(tid, core);
    }
}
