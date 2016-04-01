use fnv::FnvHasher;

use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::collections::hash_map::Iter;
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::ops::AddAssign;

use utils::{Flow, flow_hash};

type FnvHash = BuildHasherDefault<FnvHasher>;
const VEC_SIZE: usize = 1 << 20;

/// A generic store for associating some merge-able type with each flow. Note, the merge must be commutative, we do not
/// guarantee ordering for things being merged. The merge function is implemented by implementing the
/// (AddAssign)[https://doc.rust-lang.org/std/ops/trait.AddAssign.html] trait and overriding the `add_assign` method
/// there. We assume that the stored quantity needs to only be accessed from the control plane, and cannot be accessed
/// from the data plane.
///
/// #[FIXME]
/// Garbage collection.
// FIXME: Consider using a channel
#[derive(Clone)]
pub struct MergeableStoreDataPath<T: AddAssign<T> + Default + Clone> {
    flow_counters: Vec<T>,
    /// Contains the counts on the data path.
    cache: Vec<(Flow, T)>,
    /// How many updates has this counter seen.
    updates: usize,
    /// How many updates to see before sending.
    delay: usize,
    channel: SyncSender<Vec<(Flow, T)>>,
}

pub struct MergeableStoreControlPlane<T: AddAssign<T> + Default + Clone> {
    /// The actual values.
    flow_counters: HashMap<Flow, T, FnvHash>,
    channel: Receiver<Vec<(Flow, T)>>,
}

impl<T: AddAssign<T> + Default + Clone> MergeableStoreDataPath<T> {
    #[inline]
    fn update_cache(&mut self) {
        for (f, v) in self.cache.clone() {
            self.flow_counters[flow_hash(&f) as usize % VEC_SIZE] += v;
        }
    }
    /// Change the value for the given `Flow`.
    #[inline]
    pub fn update(&mut self, flow: Flow, inc: T) {
        self.cache.push((flow, inc));
        self.updates += 1;
        if self.updates >= self.delay {
            self.updates = 0;
            self.update_cache();
            match self.channel.try_send(self.cache.drain(0..).collect()) {
                Ok(_) => (),
                Err(_) => (),
            };
        }
    }

    #[inline]
    pub fn get(&self, flow: &Flow) -> T {
        self.flow_counters[flow_hash(&flow) as usize % VEC_SIZE].clone()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.flow_counters.len()
    }

    /// Remove an entry from the table.
    #[inline]
    pub fn remove(&mut self, flow: &Flow) -> T {
        let val = self.flow_counters[flow_hash(&flow) as usize % VEC_SIZE].clone();
        self.flow_counters[flow_hash(&flow) as usize % VEC_SIZE] = Default::default();
        val
    }
}

impl<T: AddAssign<T> + Default + Clone> MergeableStoreControlPlane<T> {
    fn update_internal(&mut self, v: Vec<(Flow, T)>) {
        for (flow, c) in v {
            *(self.flow_counters.entry(flow).or_insert(Default::default())) += c;
        }
    }

    /// Call periodically to drain the queue.
    pub fn recv(&mut self) {
        match self.channel.try_recv() {
            Err(_) => (),
            Ok(v) => self.update_internal(v),
        }
    }

    #[inline]
    pub fn get(&self, flow: &Flow) -> T {
        match self.flow_counters.get(flow) {
            Some(i) => i.clone(),
            None => Default::default(),
        }
    }

    #[inline]
    pub fn iter(&self) -> Iter<Flow, T> {
        self.flow_counters.iter()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.flow_counters.len()
    }

    /// Remove an entry from the table.
    #[inline]
    pub fn remove(&mut self, flow: &Flow) -> T {
        self.flow_counters.remove(flow).unwrap_or(Default::default())
    }
}

/// Create a MergeableStore. `delay` specifies the number of buckets buffered together, while `channel_size`
/// specifies the number of outstanding messages.
pub fn new_mergeable_store<T: AddAssign<T> + Default + Clone>(delay: usize,
                                                                 channel_size: usize)
                                                                 -> (MergeableStoreDataPath<T>,
                                                                     Box<MergeableStoreControlPlane<T>>) {
    let (sender, receiver) = sync_channel(channel_size);
    (MergeableStoreDataPath {
        cache: Vec::with_capacity(delay),
        // FIXME: Maybe cuckoo hash this?
        flow_counters: vec![Default::default(); VEC_SIZE],
        updates: 0,
        delay: delay,
        channel: sender,
    },
     box MergeableStoreControlPlane {
        // FIXME: Don't need this to be quite this big?
        flow_counters: HashMap::with_capacity_and_hasher(VEC_SIZE, Default::default()),
        channel: receiver,
    })
}
