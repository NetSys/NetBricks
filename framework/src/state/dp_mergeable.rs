use fnv::FnvHasher;

use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::ops::AddAssign;
use std::collections::hash_map::Iter;

use utils::Flow;

/// A generic store for associating some merge-able type with each flow. Note, the merge must be commutative, we do not
/// guarantee ordering for things being merged. The merge function is implemented by implementing the
/// (AddAssign)[https://doc.rust-lang.org/std/ops/trait.AddAssign.html] trait and overriding the `add_assign` method
/// there. We assume that the quantity stored here does not need to be accessed by the control plane.
///
/// #[FIXME]
/// Garbage collection.
type FnvHash = BuildHasherDefault<FnvHasher>;
const VEC_SIZE: usize = 1<<24;
#[derive(Clone)]
pub struct DpMergeableStore<T: AddAssign<T> + Default> {
    /// Contains the counts on the data path.
    flow_counters: HashMap<Flow, T, FnvHash>,
}

impl<T: AddAssign<T> + Default> DpMergeableStore<T> {
    pub fn with_size(size: usize) -> DpMergeableStore<T> {
        DpMergeableStore {
            flow_counters: HashMap::with_capacity_and_hasher(size, Default::default()),
        }
    }

    pub fn new() -> DpMergeableStore<T> {
        DpMergeableStore::with_size(VEC_SIZE)
    }

    /// Change the value for the given `Flow`.
    #[inline]
    pub fn update(&mut self, flow: Flow, inc: T) {
        let entry = self.flow_counters.entry(flow).or_insert(Default::default());
        *entry += inc;
        //let val = entry;
        //*entry = val + inc;
    }

    /// Remove an entry from the table.
    #[inline]
    pub fn remove(&mut self, flow: &Flow) -> T {
        self.flow_counters.remove(flow).unwrap_or(Default::default())
    }

    /// Iterate over all the stored entries. This is a bit weird to do in the data plane.
    /// 
    /// #[Warning]
    /// This might have severe performance penalties.
    pub fn iter(&self) -> Iter<Flow, T> {
        self.flow_counters.iter()
    }
}
