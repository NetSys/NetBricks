use fnv::FnvHasher;

use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::sync::mpsc::*;

use utils::Flow;

/// A generic store for associating a counter with each flow. This one assumes the counts are only ready by the control
/// plane (which allows us to improve datapath performance).
type FnvHash = BuildHasherDefault<FnvHasher>;
const VEC_SIZE: usize = 1<<24;
#[derive(Clone)]
pub struct ControlPlaneCounterDataPath {
    /// Contains the counts on the data path.
    cache: Vec<(Flow, isize)>,
    /// How many updates has this counter seen.
    updates: usize,
    /// How many updates to see before sending.
    delay: usize,
    channel: SyncSender<Vec<(Flow, isize)>>,
}

pub struct ControlPlaneCounterControlPlane {
    /// The actual values.
    flow_counters: HashMap<Flow, isize, FnvHash>,
    channel: Receiver<Vec<(Flow, isize)>>,
}

impl ControlPlaneCounterDataPath {
    /// Change the value for the given `Flow` in the flowcounter.
    #[inline]
    pub fn update(&mut self, flow: Flow, inc: isize) {
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

impl ControlPlaneCounterControlPlane {
    fn update_internal(&mut self, v: Vec<(Flow, isize)>) {
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

    pub fn get(&self, flow: &Flow) -> isize {
        match self.flow_counters.get(flow) {
            Some(i) => *i,
            None => 0,
        }
    }

    pub fn iter(&self) -> Iter<Flow, isize> {
        self.flow_counters.iter()
    }
}

/// Create a ControlPlaneCounter. `delay` specifies the number of buckets buffered together, while `channel_size`
/// specifies the number of outstanding messages.
pub fn new_cp_flow_counter(delay: usize, channel_size: usize) ->
    (ControlPlaneCounterDataPath, Box<ControlPlaneCounterControlPlane>) {
    let (sender, receiver) = sync_channel(channel_size);
    (ControlPlaneCounterDataPath {
       cache: Vec::with_capacity(delay),
       updates: 0,
       delay: delay,
       channel: sender,
    },
    box ControlPlaneCounterControlPlane {
       // FIXME: Don't need this to be quite this big?
        flow_counters: HashMap::with_capacity_and_hasher(VEC_SIZE, Default::default()),
        channel: receiver,
    })
}
