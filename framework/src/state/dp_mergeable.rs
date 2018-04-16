use fnv::FnvHasher;
use std::collections::hash_map::Iter;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::ops::AddAssign;
use utils::Flow;

/// A generic store for associating some merge-able type with each flow. Note, the merge must be commutative, we do not
/// guarantee ordering for things being merged. The merge function is implemented by implementing the
/// [`AddAssign`](https://doc.rust-lang.org/std/ops/trait.AddAssign.html) trait and overriding the `add_assign` method
/// there. We assume that the quantity stored here does not need to be accessed by the control plane and can only be
/// accessed from the data plane. The `cache_size` should be tuned depending on whether gets or puts are the most common
/// operation in this table.
///
/// #[FIXME]
/// Garbage collection.
type FnvHash = BuildHasherDefault<FnvHasher>;
const VEC_SIZE: usize = 1 << 24;
#[derive(Clone)]
pub struct DpMergeableStore<T: AddAssign<T> + Default> {
    /// Contains the counts on the data path.
    state: HashMap<Flow, T, FnvHash>,
    cache: Vec<(Flow, T)>,
    cache_size: usize,
}

const CACHE_SIZE: usize = 1 << 14;
impl<T: AddAssign<T> + Default> DpMergeableStore<T> {
    pub fn with_cache_and_size(cache: usize, size: usize) -> DpMergeableStore<T> {
        DpMergeableStore {
            state: HashMap::with_capacity_and_hasher(size, Default::default()),
            cache: Vec::with_capacity(cache),
            cache_size: cache,
        }
    }

    pub fn new() -> DpMergeableStore<T> {
        DpMergeableStore::with_cache_and_size(CACHE_SIZE, VEC_SIZE)
    }

    fn merge_cache(&mut self) {
        self.state.extend(self.cache.drain(0..));
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
        self.merge_cache();
        self.state.remove(flow).unwrap_or_else(Default::default)
    }

    /// Iterate over all the stored entries. This is a bit weird to do in the data plane.
    ///
    /// #[Warning]
    /// This might have severe performance penalties.
    pub fn iter(&mut self) -> Iter<Flow, T> {
        self.merge_cache();
        self.state.iter()
    }

    /// Length of the table.
    pub fn len(&mut self) -> usize {
        self.merge_cache();
        self.state.len()
    }

    /// Is table empty
    pub fn is_empty(&self) -> bool {
        self.state.is_empty() && self.cache.is_empty()
    }
}
