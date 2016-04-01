use twox_hash::XxHash;

use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::collections::hash_map::Iter;
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::ops::AddAssign;

use utils::Flow;

//type FnvHash = BuildHasherDefault<FnvHasher>;
type XxHasher = BuildHasherDefault<XxHash>;
const VEC_SIZE: usize = 1 << 24;

/// A generic store for associating some merge-able type with each flow. Note, the merge must be commutative, we do not
/// guarantee ordering for things being merged. The merge function is implemented by implementing the
/// (AddAssign)[https://doc.rust-lang.org/std/ops/trait.AddAssign.html] trait and overriding the `add_assign` method
/// there. We assume that the stored quantity needs to only be accessed from the control plane, and cannot be accessed
/// from the data plane.
///
/// #[FIXME]
/// Garbage collection.
#[derive(Clone)]
pub struct CpMergeableStoreDataPath<T: AddAssign<T> + Default + Clone> {
    /// Contains the counts on the data path.
    cache: Vec<(Flow, T)>,
    /// How many updates has this counter seen.
    updates: usize,
    /// How many updates to see before sending.
    delay: usize,
    channel: SyncSender<Vec<(Flow, T)>>,
}

pub struct CpMergeableStoreControlPlane<T: AddAssign<T> + Default + Clone> {
    /// The actual values.
    flow_counters: HashMap<Flow, T, XxHasher>,
    channel: Receiver<Vec<(Flow, T)>>,
}

impl<T: AddAssign<T> + Default + Clone> CpMergeableStoreDataPath<T> {
    /// Change the value for the given `Flow`.
    #[inline]
    pub fn update(&mut self, flow: Flow, inc: T) {
        self.cache.push((flow, inc));
        self.updates += 1;
        if self.updates >= self.delay {
            self.updates = 0;
            match self.channel.try_send(self.cache.drain(0..).collect()) {
                Ok(_) => (),
                Err(_) => (),
            };
        }
    }
}

impl<T: AddAssign<T> + Default + Clone> CpMergeableStoreControlPlane<T> {
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

    /// Remove an entry from the table.
    #[inline]
    pub fn remove(&mut self, flow: &Flow) -> T {
        self.flow_counters.remove(flow).unwrap_or(Default::default())
    }
}

/// Create a CpMergeableStore. `delay` specifies the number of buckets buffered together, while `channel_size`
/// specifies the number of outstanding messages.
pub fn new_cp_mergeable_store<T: AddAssign<T> + Default + Clone>(delay: usize,
                                                                 channel_size: usize)
                                                                 -> (CpMergeableStoreDataPath<T>,
                                                                     Box<CpMergeableStoreControlPlane<T>>) {
    let (sender, receiver) = sync_channel(channel_size);
    (CpMergeableStoreDataPath {
        cache: Vec::with_capacity(delay),
        updates: 0,
        delay: delay,
        channel: sender,
    },
     box CpMergeableStoreControlPlane {
        // FIXME: Don't need this to be quite this big?
        flow_counters: HashMap::with_capacity_and_hasher(VEC_SIZE, Default::default()),
        channel: receiver,
    })
}
