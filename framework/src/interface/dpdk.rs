use super::METADATA_SLOTS;
use super::native_include as ldpdk;
use config::{DEFAULT_CACHE_SIZE, DEFAULT_POOL_SIZE, NetbricksConfiguration};
use native::libnuma;
use native::zcsi;
use std::cell::Cell;
use std::ffi::CString;
use std::ptr;
use std::iter;

/// Initialize the system, whitelisting some set of NICs and allocating mempool of given size.
fn init_system_wl_with_mempool(name: &str, core: i32, devices: &[String], pool_size: u32, cache_size: u32) {
    let name_cstr = CString::new(name).unwrap();

    //let pci_cstr: Vec<_> = pci.iter().map(|p| CString::new(&p[..]).unwrap()).collect();
    //let mut whitelist: Vec<_> = pci_cstr.iter().map(|p| p.as_ptr()).collect();
    let mut dpdk_args = vec![];
    //let core_str = CString::new(format!("0x{:x}", 1u32 << core)).unwrap();
    // FIXME: Maybe replace with placement syntax
    // First we need to push in name.
    unsafe {
        dpdk_args.push(name_cstr.into_raw());
        dpdk_args.push(CString::new("--master-lcore").unwrap().into_raw());
        // Using RTE_MAX_LCORE as core ID for master.
        let master_lcore = ldpdk::RTE_MAX_LCORE - 1;
        dpdk_args.push(CString::new(master_lcore.to_string()).unwrap().into_raw());
        dpdk_args.push(CString::new(format!("{}@{}", master_lcore, core)).unwrap().into_raw());
        dpdk_args.push(CString::new("--no-shconf").unwrap().into_raw());

        // Fix this
        let numa_nodes = 2;
        let mem_vec : Vec<_> = iter::repeat(pool_size.to_string()).take(numa_nodes).collect();
        let mem = mem_vec.join(",");
        dpdk_args.push(CString::new("--socket-mem").unwrap().into_raw());
        dpdk_args.push(CString::new(mem).unwrap().into_raw());
        dpdk_args.push(CString::new("--huge-unlink").unwrap().into_raw());
        // White list a fake card so everything is blacklisted by default.
        dpdk_args.push(CString::new("-w").unwrap().into_raw());
        dpdk_args.push(CString::new("99:99.0").unwrap().into_raw());
        for dev in devices {
            dpdk_args.push(CString::new("-w").unwrap().into_raw());
            dpdk_args.push(CString::new(&dev[..]).unwrap().into_raw());
        }
        dpdk_args.push(ptr::null_mut());
        let arg_len = dpdk_args.len() as i32;
        let ret = ldpdk::rte_eal_init(arg_len, dpdk_args.as_mut_ptr());
        if ret != 0 {
            panic!("Could not initialize DPDK -- errno {}", -ret)
        }
    }
}

/// Initialize the system, whitelisting some set of NICs.
pub fn init_system_wl(name: &str, core: i32, pci: &[String]) {
    init_system_wl_with_mempool(name, core, pci, DEFAULT_POOL_SIZE, DEFAULT_CACHE_SIZE);
    set_numa_domain();
}

/// Initialize the system as a DPDK secondary process with a set of VDEVs. User must specify mempool name to use.
pub fn init_system_secondary(name: &str, core: i32) {
    let name_cstr = CString::new(name).unwrap();
    let mut vdev_list = vec![];
    unsafe {
        let ret = zcsi::init_secondary(
            name_cstr.as_ptr(),
            name.len() as i32,
            core,
            vdev_list.as_mut_ptr(),
            0,
        );
        if ret != 0 {
            panic!("Could not initialize secondary process errno {}", ret)
        }
    }
    set_numa_domain();
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
        init_system_wl_with_mempool(
            &config.name[..],
            config.primary_core,
            &[],
            config.pool_size,
            config.cache_size,
        );
    }
    set_numa_domain();
}

thread_local!(static NUMA_DOMAIN: Cell<i32> = Cell::new(-1));

fn set_numa_domain() {
    let domain = unsafe {
        if libnuma::numa_available() == -1 {
            println!("No NUMA information found, support disabled");
            -1
        } else {
            let domain = libnuma::numa_preferred();
            println!("Running on node {}", domain);
            domain
        }
    };
    NUMA_DOMAIN.with(|f| f.set(domain))
}

/// Affinitize a pthread to a core and assign a DPDK thread ID.
pub fn init_thread(tid: i32, core: i32) {
    let numa = unsafe { zcsi::init_thread(tid, core) };
    NUMA_DOMAIN.with(|f| {
        f.set(numa);
    });
    if numa == -1 {
        println!("No NUMA information found, support disabled");
    } else {
        println!("Running on node {}", numa);
    };
}

#[inline]
pub fn get_domain() -> i32 {
    NUMA_DOMAIN.with(|f| f.get())
}
