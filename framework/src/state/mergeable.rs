use fnv::FnvHasher;
use packets::ip::Flow;
use std::cmp::max;
use std::collections::hash_map::Iter;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::ops::AddAssign;
use std::sync::{Arc, RwLock, RwLockReadGuard};

/// A generic store for associating some merge-able type with each flow. Note,
/// the merge must be commutative, we do not guarantee ordering for things being
/// merged. The merge function is implemented by implementing the
/// [`AddAssign`](https://doc.rust-lang.org/std/ops/trait.AddAssign.html) trait
/// and overriding the `add_assign` method there. We assume that the quantity
/// stored here does not need to be accessed by the control plane and can only
/// be accessed from the data plane. The `cache_size` should be tuned depending
/// on whether gets or puts are the most common operation in this table.
///
/// #[FIXME]
/// Garbage collection.
/// The current version does not work well with large flow tables. The problem
/// is we need to record a set of differences rather than copying the entire
/// hashmap. This of course comes with some consistency issues, so we need to
/// fix this.
type FnvHash = BuildHasherDefault<FnvHasher>;
const VEC_SIZE: usize = 1 << 10;
const CACHE_SIZE: usize = 1 << 10;
const MAX_CACHE_SIZE: usize = 1 << 20;
const CHAN_SIZE: usize = 128;

#[derive(Default)]
pub struct MergeableStoreCP<T: AddAssign<T> + Default + Clone> {
    flow_counters: HashMap<Flow, T, FnvHash>,
    hashmaps: Vec<Arc<RwLock<HashMap<Flow, T, FnvHash>>>>,
}

impl<T: AddAssign<T> + Default + Clone> MergeableStoreCP<T> {
    pub fn new() -> MergeableStoreCP<T> {
        MergeableStoreCP {
            flow_counters: HashMap::with_capacity_and_hasher(VEC_SIZE << 6, Default::default()),
            hashmaps: Vec::with_capacity(CHAN_SIZE),
        }
    }

    pub fn dp_store_with_cache_and_size(
        &mut self,
        cache: usize,
        size: usize,
    ) -> MergeableStoreDP<T> {
        let hmap = Arc::new(RwLock::new(HashMap::with_capacity_and_hasher(
            size,
            Default::default(),
        )));
        self.hashmaps.push(hmap.clone());
        MergeableStoreDP {
            flow_counters: hmap,
            cache: Vec::with_capacity(cache),
            cache_size: cache,
            base_cache_size: cache,
            len: 0,
        }
    }

    pub fn dp_store(&mut self) -> MergeableStoreDP<T> {
        MergeableStoreCP::dp_store_with_cache_and_size(self, CACHE_SIZE, VEC_SIZE)
    }

    fn hmap_to_vec(hash: &RwLockReadGuard<HashMap<Flow, T, FnvHash>>) -> Vec<(Flow, T)> {
        let mut t = Vec::with_capacity(hash.len());
        t.extend(hash.iter().map(|(f, v)| (*f, v.clone())));
        t
    }

    pub fn sync(&mut self) {
        let mut copies: Vec<Vec<_>> = Vec::with_capacity(self.hashmaps.len());
        {
            for hmap in &self.hashmaps {
                {
                    if let Ok(g) = hmap.try_read() {
                        copies.push(MergeableStoreCP::hmap_to_vec(&g));
                    }
                }
            }
        }
        self.flow_counters.clear();
        for mut copy in copies {
            self.flow_counters.extend(copy.drain(0..));
        }
    }

    pub fn get(&self, flow: &Flow) -> T {
        match self.flow_counters.get(flow) {
            Some(i) => i.clone(),
            None => Default::default(),
        }
    }

    pub fn iter(&self) -> Iter<Flow, T> {
        self.flow_counters.iter()
    }

    pub fn len(&self) -> usize {
        self.flow_counters.len()
    }

    pub fn is_empty(&self) -> bool {
        self.flow_counters.is_empty()
    }
}

#[derive(Clone)]
pub struct MergeableStoreDP<T: AddAssign<T> + Default + Clone> {
    /// Contains the counts on the data path.
    flow_counters: Arc<RwLock<HashMap<Flow, T, FnvHash>>>,
    cache: Vec<(Flow, T)>,
    base_cache_size: usize,
    cache_size: usize,
    len: usize,
}

impl<T: AddAssign<T> + Default + Clone> MergeableStoreDP<T> {
    fn merge_cache(&mut self) {
        match self.flow_counters.try_write() {
            Ok(mut g) => {
                g.extend(self.cache.drain(0..));
                self.cache_size = self.base_cache_size;
                self.len = g.len();
            }
            _ => self.cache_size = max(self.cache_size * 2, MAX_CACHE_SIZE),
        }
    }

    /// Change the value for the given `Flow`.
    #[inline]
    pub fn update(&mut self, flow: Flow, inc: T) {
        {
            self.cache.push((flow, inc));
        }
        if self.cache.len() >= self.cache_size {
            self.merge_cache();
        }
    }

    /// Remove an entry from the table.
    #[inline]
    pub fn remove(&mut self, flow: &Flow) -> T {
        // self.merge_cache();
        match self.flow_counters.write() {
            Ok(mut g) => {
                g.extend(self.cache.drain(0..));
                self.cache_size = self.base_cache_size;
                self.len = g.len();
                g.remove(flow).unwrap_or_else(Default::default)
            }
            _ => panic!("Could not acquire write lock"),
        }
    }

    /// Approximate length of the table.
    pub fn len(&mut self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len != 0
    }
}
