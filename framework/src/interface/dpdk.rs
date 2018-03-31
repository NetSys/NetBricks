use super::METADATA_SLOTS;
use super::native_include as ldpdk;
use config::{NetbricksConfiguration, DEFAULT_CACHE_SIZE, DEFAULT_POOL_SIZE};
use libc;
use native::libnuma;
use native::zcsi as lzcsi;
use std::cell::Cell;
use std::cmp;
use std::ffi::CString;
use std::io::Error;
use std::iter;
use std::mem;
use std::ptr;

unsafe fn init_socket_mempool(
    socket: i32,
    pool_size: u32,
    cache_size: u32,
    metadata_slots: u16,
) -> *mut ldpdk::rte_mempool {
    let name = CString::new(format!("pframe{}", socket)).unwrap();
    ldpdk::rte_pktmbuf_pool_create(
        name.into_raw(),
        pool_size,
        cache_size,
        metadata_slots * 64,
        ldpdk::RTE_MBUF_DEFAULT_BUF_SIZE as u16,
        socket,
    )
}

/// Call into libnuma to bind thread to NUMA node.
unsafe fn bind_thread_to_numa_node(socket: u32) {
    let bitmask = libnuma::numa_bitmask_setbit(
        libnuma::numa_bitmask_clearall(libnuma::numa_bitmask_alloc(libnuma::numa_num_possible_nodes() as u32)),
        socket,
    );
    libnuma::numa_bind(bitmask);
}

/// Initialize the system, whitelisting some set of NICs and allocating mempool of given size.
fn init_system_wl_with_mempool(name: &str, core: i32, devices: &[String], pool_size: u32, cache_size: u32) {
    let name_cstr = CString::new(name).unwrap();

    let mut dpdk_args = vec![];
    // FIXME: Maybe replace with placement syntax
    unsafe {
        // First we need to push in name.
        dpdk_args.push(name_cstr.into_raw());
        dpdk_args.push(CString::new("--master-lcore").unwrap().into_raw());
        // Using RTE_MAX_LCORE as core ID for master.
        let master_lcore = ldpdk::RTE_MAX_LCORE - 1;
        dpdk_args.push(CString::new(master_lcore.to_string()).unwrap().into_raw());
        dpdk_args.push(CString::new("--lcore").unwrap().into_raw());
        dpdk_args.push(CString::new(format!("{}@{}", master_lcore, core)).unwrap().into_raw());
        dpdk_args.push(CString::new("--no-shconf").unwrap().into_raw());

        let numa_available = libnuma::numa_available();
        let numa_nodes = if numa_available == -1 {
            // FIXME: Warn.
            1
        } else {
            // The cmp::max is to take care of any cases where libnuma is broken.
            cmp::max(1, libnuma::numa_num_configured_nodes())
        } as usize;
        let mem_vec: Vec<_> = iter::repeat(pool_size.to_string()).take(numa_nodes).collect();
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
        let arg_len = dpdk_args.len() as i32;
        println!("arg_len = {}", arg_len);
        dpdk_args.push(ptr::null_mut());
        let ret = ldpdk::rte_eal_init(arg_len, dpdk_args.as_mut_ptr());
        if ret == -1 {
            panic!(
                "Could not initialize DPDK -- errno {:?}",
                Error::from_raw_os_error(-ret)
            )
        }

        if numa_available != -1 {
            let socket = ldpdk::lcore_config[master_lcore as usize].socket_id;
            bind_thread_to_numa_node(socket);
        }

        for sock in 0..numa_nodes {
            let _ = init_socket_mempool(sock as i32, pool_size, cache_size, METADATA_SLOTS as u16);
        }
        set_lcore_id(master_lcore as i32);
    }
}

/// Initialize the system, whitelisting some set of NICs.
pub fn init_system_wl(name: &str, core: i32, pci: &[String]) {
    init_system_wl_with_mempool(name, core, pci, DEFAULT_POOL_SIZE, DEFAULT_CACHE_SIZE);
}

/// Initialize the system based on the supplied scheduler configuration.
pub fn init_system(config: &NetbricksConfiguration) {
    if config.name.is_empty() {
        panic!("Configuration must provide a name.");
    }
    // We init with all devices blacklisted and rely on the attach logic to white list them as necessary.
    init_system_wl_with_mempool(
        &config.name[..],
        config.primary_core,
        &[],
        config.pool_size,
        config.cache_size,
    );
}

thread_local!(static CURRENT_LCORE: Cell<i32> = Cell::new(-1));

fn set_lcore_id(lcore: i32) {
    CURRENT_LCORE.with(|f| f.set(lcore))
}

/// Affinitize a pthread to a core and assign a DPDK thread ID.
pub fn init_thread(tid: i32, core: i32) {
    //unsafe { zcsi::init_thread(tid, core) };
    unsafe {
        let mut cset: libc::cpu_set_t = mem::uninitialized();
        libc::CPU_ZERO(&mut cset);
        libc::CPU_SET(core as usize, &mut cset);
        {
            let mut cset_dpdk = mem::transmute(cset);
            let cset = &mut cset_dpdk as *mut ldpdk::cpu_set_t;
            ldpdk::rte_thread_set_affinity(cset);
        }
        let numa_available = libnuma::numa_available();
        let socket = ldpdk::lcore_config[core as usize].socket_id;
        if numa_available != -1 {
            bind_thread_to_numa_node(socket)
        };
        // FIXME: Need to set lcore_id for use by mempool caches, which are accessed by some PMD drivers
        let init_result = lzcsi::init_thread(tid, core);
        if init_result != 1 {
            panic!("init_thread failed")
        }
    }
    set_lcore_id(tid)
}

#[inline]
pub fn get_domain() -> u32 {
    let lcore = CURRENT_LCORE.with(|f| f.get());
    unsafe { ldpdk::lcore_config[lcore as usize].socket_id }
}
