use libc::c_void;
use std::cmp::{Eq, PartialEq};

#[link(name = "numa")]
#[allow(dead_code)]
extern "C" {
    /// Check if NUMA support is enabled. Returns -1 if not enabled, in which case other functions will
    /// panic.
    pub fn numa_available() -> i32;
    /// The number of NUMA nodes supported by the system.
    pub fn numa_max_possible_node() -> i32;
    /// Size of NUMA mask in kernel.
    pub fn numa_num_possible_nodes() -> i32;
    /// Size of CPU mask in kernel.
    pub fn numa_num_possible_cpus() -> i32;
    /// The number of NUMA nodes available in the system.
    pub fn numa_max_node() -> i32;
    /// The number of memory nodes configured in the system.
    pub fn numa_num_configured_nodes() -> i32;
    /// The number of configured CPUs in the system
    pub fn numa_num_configured_cpus() -> i32;
    /// Number of CPUs current task can use.
    pub fn numa_num_task_cpus() -> i32;
    /// Number of nodes current task can use.
    pub fn numa_num_task_nodes() -> i32;

    pub fn numa_node_size(node: i32, freep: *mut u64) -> u64;

    pub fn numa_preferred() -> i32;
    pub fn numa_set_preferred(node: i32);
    pub fn numa_get_interleave_node() -> i32;
    // numa_interleave_memory: Leaving this aside until I figure out a safe way/reason to use this.
    pub fn numa_set_localalloc();

    pub fn numa_run_on_node(node: i32);

    pub fn numa_alloc_onnode(size: usize, node: i32) -> *mut c_void;
    pub fn numa_alloc_local(size: usize) -> *mut c_void;
    pub fn numa_alloc_interleaved(size: usize) -> *mut c_void;
    pub fn numa_alloc(size: usize) -> *mut c_void;
    pub fn numa_realloc(addr: *mut c_void, old_size: usize, new_size: usize) -> *mut c_void;
    pub fn numa_free(addr: *mut c_void, size: usize);

    pub fn numa_tonode_memory(start: *mut c_void, size: usize, node: u32);
    pub fn numa_setlocal_memory(start: *mut c_void, size: usize);
    pub fn numa_police_memory(start: *mut c_void, size: usize);
    pub fn numa_set_bind_policy(strict: i32);
    pub fn numa_set_strict(strict: i32);
}

mod wrapped {
    use libc::c_void;
    use native::libnuma::NumaBitmap;
    extern "C" {
        /// Memory nodes from which allocation is permitted.
        pub fn numa_get_mems_allowed() -> *mut NumaBitmap;

        pub fn numa_get_interleave_mask() -> *mut NumaBitmap;
        pub fn numa_set_interleave_mask(bitmap: *mut NumaBitmap);

        pub fn numa_bind(bitmap: *mut NumaBitmap);

        pub fn numa_set_membind(bitmap: *mut NumaBitmap);
        pub fn numa_get_membind() -> *mut NumaBitmap;

        pub fn numa_run_on_node_mask(mask: *mut NumaBitmap);
        pub fn numa_get_run_node_mask() -> *mut NumaBitmap;

        pub fn numa_alloc_interleaved_subset(size: usize, mask: *mut NumaBitmap) -> *mut c_void;
        pub fn numa_distance(node1: i32, node2: i32) -> i32;

        pub fn numa_bitmask_alloc(bits: u32) -> *mut NumaBitmap;
        pub fn numa_bitmask_free(mask: *mut NumaBitmap);

        pub fn numa_bitmask_clearall(mask: *mut NumaBitmap) -> *mut NumaBitmap;
        pub fn numa_bitmask_clearbit(mask: *mut NumaBitmap, bit: u32) -> *mut NumaBitmap;
        pub fn numa_bitmask_equal(mask1: *const NumaBitmap, mask2: *const NumaBitmap) -> i32;
        pub fn numa_bitmask_isbitset(mask: *const NumaBitmap, n: u32) -> i32;
        pub fn numa_bitmask_nbytes(mask: *mut NumaBitmap) -> u32;
        pub fn numa_bitmask_setall(mask: *mut NumaBitmap) -> *mut NumaBitmap;
        pub fn numa_bitmask_setbit(mask: *mut NumaBitmap, bit: u32) -> *mut NumaBitmap;
        pub fn numa_tonodemask_memory(start: *mut c_void, size: usize, mask: *mut NumaBitmap);
    }
}

#[repr(C)]
pub struct NumaBitmap {
    size: usize,
    mask: *mut u64,
}

pub struct Bitmask {
    pub bitmask: *mut NumaBitmap,
}

impl PartialEq for Bitmask {
    fn eq(&self, other: &Bitmask) -> bool {
        unsafe { wrapped::numa_bitmask_equal(self.bitmask, other.bitmask) == 0 }
    }
}

impl Eq for Bitmask {}

impl Bitmask {
    pub unsafe fn allocate_node_mask() -> Bitmask {
        Bitmask {
            bitmask: wrapped::numa_bitmask_clearall(wrapped::numa_bitmask_alloc(
                numa_num_possible_nodes() as u32,
            )),
        }
    }

    pub unsafe fn allocate_cpu_mask() -> Bitmask {
        Bitmask {
            bitmask: wrapped::numa_bitmask_clearall(wrapped::numa_bitmask_alloc(
                numa_num_possible_cpus() as u32,
            )),
        }
    }

    #[inline]
    fn assert_size(&self, bit: usize) {
        unsafe {
            assert!(bit < (*self.bitmask).size);
        }
    }

    pub fn clear_bit(&mut self, bit: usize) {
        self.assert_size(bit);
        unsafe { wrapped::numa_bitmask_clearbit(self.bitmask, bit as u32) };
    }

    pub fn clear(&mut self) {
        unsafe { wrapped::numa_bitmask_clearall(self.bitmask) };
    }

    pub fn get_mems_allowed() -> Bitmask {
        Bitmask {
            bitmask: unsafe { wrapped::numa_get_mems_allowed() },
        }
    }

    pub fn get_interleaved_mask() -> Bitmask {
        Bitmask {
            bitmask: unsafe { wrapped::numa_get_interleave_mask() },
        }
    }

    pub fn get_membind() -> Bitmask {
        Bitmask {
            bitmask: unsafe { wrapped::numa_get_membind() },
        }
    }

    pub fn get_run_node_mask() -> Bitmask {
        Bitmask {
            bitmask: unsafe { wrapped::numa_get_run_node_mask() },
        }
    }

    /// Set bit in bitmask.
    pub fn set_bit(&mut self, bit: usize) {
        self.assert_size(bit);
        unsafe { wrapped::numa_bitmask_setbit(self.bitmask, bit as u32) };
    }

    /// Set all bits.
    pub fn set_all(&mut self) {
        unsafe { wrapped::numa_bitmask_setall(self.bitmask) };
    }

    pub fn bit(&self, idx: usize) -> bool {
        self.assert_size(idx);
        unsafe { wrapped::numa_bitmask_isbitset(self.bitmask, idx as u32) == 1 }
    }

    pub fn size_in_bytes(&self) -> usize {
        let size = unsafe { wrapped::numa_bitmask_nbytes(self.bitmask) };
        size as usize
    }
}

impl Drop for Bitmask {
    fn drop(&mut self) {
        unsafe { wrapped::numa_bitmask_free(self.bitmask) }
    }
}

pub fn numa_distance(node1: i32, node2: i32) -> i32 {
    unsafe { wrapped::numa_distance(node1, node2) }
}

pub unsafe fn numa_set_interleave_mask(bitmap: &mut Bitmask) {
    wrapped::numa_set_interleave_mask(bitmap.bitmask)
}

pub unsafe fn numa_bind(bitmap: &mut Bitmask) {
    wrapped::numa_bind(bitmap.bitmask)
}

pub unsafe fn numa_set_membind(bitmap: &mut Bitmask) {
    wrapped::numa_set_membind(bitmap.bitmask)
}

pub unsafe fn numa_run_on_node_mask(mask: &mut Bitmask) {
    wrapped::numa_run_on_node_mask(mask.bitmask)
}

pub unsafe fn numa_alloc_interleaved_subset(size: usize, mask: &mut Bitmask) -> *mut c_void {
    wrapped::numa_alloc_interleaved_subset(size, mask.bitmask)
}

pub unsafe fn numa_tonodemask_memory(start: *mut c_void, size: usize, mask: &mut Bitmask) {
    wrapped::numa_tonodemask_memory(start, size, mask.bitmask)
}
