use fnv::FnvHasher;

use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::ops::AddAssign;
use std::collections::hash_map::Iter;
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};

use utils::Flow;

/// A generic store for associating some merge-able type with each flow. Note, the merge must be commutative, we do not
/// guarantee ordering for things being merged. The merge function is implemented by implementing the
/// [AddAssign](https://doc.rust-lang.org/std/ops/trait.AddAssign.html) trait and overriding the `add_assign` method
/// there. We assume that the quantity stored here does not need to be accessed by the control plane and can only be
/// accessed from the data plane. The `cache_size` should be tuned depending on whether gets or puts are the most common
/// operation in this table.
///
/// #[FIXME]
/// Garbage collection.
type FnvHash = BuildHasherDefault<FnvHasher>;
const VEC_SIZE: usize = 1 << 24;
const CACHE_SIZE: usize = 1 << 14;
const CHAN_SIZE: usize = 128;

pub struct MergeableStoreCP<T: AddAssign<T> + Default + Clone> {
    response_chan: Receiver<Vec<(Flow, T)>>,
    response_sender: SyncSender<Vec<(Flow, T)>>,
}

#[derive(Clone)]
pub struct MergeableStoreDP<T: AddAssign<T> + Default + Clone> {
    /// Contains the counts on the data path.
    flow_counters: HashMap<Flow, T, FnvHash>,
    cache: Vec<(Flow, T)>,
    cache_size: usize,
    send_chan: SyncSender<Vec<(Flow, T)>>,
}

impl<T: AddAssign<T> + Default + Clone> MergeableStoreCP<T> {
    pub fn with_channel_size(chan_size: usize) -> MergeableStoreCP<T> {
        let (send, recv) = sync_channel(chan_size);
        MergeableStoreCP {
            response_chan: recv,
            response_sender: send,
        }
    }
    pub fn new() -> MergeableStoreCP<T> {
       MergeableStoreCP::with_channel_size(CHAN_SIZE)
    }

    pub fn dp_store_with_cache_and_size(&mut self, cache: usize, size: usize) -> MergeableStoreDP<T> {
        let send_resp = self.response_sender.clone();
        MergeableStoreDP {
            flow_counters: HashMap::with_capacity_and_hasher(size, Default::default()),
            cache: Vec::with_capacity(cache),
            cache_size: cache,
            send_chan: send_resp,
        }
    }

    pub fn dp_store(&mut self) -> MergeableStoreDP<T> {
        MergeableStoreCP::dp_store_with_cache_and_size(self, CACHE_SIZE, VEC_SIZE)
    }
}

impl<T: AddAssign<T> + Default + Clone> MergeableStoreDP<T> {
    fn merge_cache(&mut self) {
        let cache_copy = self.cache.clone();
        self.flow_counters.extend(self.cache.drain(0..));
        match self.send_chan.try_send(cache_copy) {
            _ => ()
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
        self.merge_cache();
        self.flow_counters.remove(flow).unwrap_or(Default::default())
    }

    /// Iterate over all the stored entries. This is a bit weird to do in the data plane.
    ///
    /// #[Warning]
    /// This might have severe performance penalties.
    pub fn iter(&mut self) -> Iter<Flow, T> {
        self.merge_cache();
        self.flow_counters.iter()
    }

    /// Length of the table.
    pub fn len(&mut self) -> usize {
        self.merge_cache();
        self.flow_counters.len()
    }
}
