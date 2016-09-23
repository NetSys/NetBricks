use std::fmt;
pub use self::config_reader::*;
mod config_reader;

/// NetBricks control configuration. In theory all applications create one of these, either through the use of
/// `read_configuration` or manually using args.
pub struct SchedulerConfiguration {
    /// Name, this is passed on to DPDK. If you want to run multiple DPDK apps, this needs to be unique per application.
    pub name: String,
    /// Should this process be run as a secondary process or a primary process?
    pub secondary: bool,
    /// Where should the main thread (for the examples this just sits around and prints packet counts) be run.
    pub primary_core: i32,
    /// A set of ports to be initialized.
    pub ports: Vec<PortConfiguration>,
    /// Memory pool size: sizing this pool is a bit complex; too big and you might affect caching behavior, too small
    /// and you limit how many packets are in your system overall.
    pub pool_size: u32,
    /// Size of the per-core mempool cache.
    pub cache_size: u32,
}

/// Create an empty `SchedulerConfiguration`, useful when initializing through arguments.
impl Default for SchedulerConfiguration {
    fn default() -> SchedulerConfiguration {
        SchedulerConfiguration {
            name: String::new(),
            pool_size: DEFAULT_POOL_SIZE,
            cache_size: DEFAULT_CACHE_SIZE,
            primary_core: 0,
            secondary: false,
            ports: vec![],
        }
    }
}

impl SchedulerConfiguration {
    /// Create a `SchedulerConfiguration` with a name.
    pub fn new_with_name(name: &str) -> SchedulerConfiguration {
        SchedulerConfiguration { name: String::from(name), ..Default::default() }
    }
}

impl fmt::Display for SchedulerConfiguration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f,
                    "Configuration: primary core: {}\n Ports:\n",
                    self.primary_core));
        for port in &self.ports {
            try!(write!(f, "\t{}\n", port))
        }
        write!(f, "")
    }
}

/// Configuration for each port (network device) in NetBricks.
#[derive(Default)]
pub struct PortConfiguration {
    /// Name. The exact semantics vary by backend. For DPDK, we allow things of the form:
    ///    <PCI ID> : Hardware device with PCI ID
    ///    dpdk:<PMD Descriptor>: PMD driver with arguments
    ///    bess:<port_name>: BESS RingVport with name.
    ///    ovs:<port_id>: OVS ring with ID.
    pub name: String,
    /// Core on which a given queue will be used.
    pub queues: Vec<i32>,
    /// Number of RX descriptors to use.
    pub rxd: i32,
    /// Number of TX descriptors to use.
    pub txd: i32,
    pub loopback: bool,
}

impl fmt::Display for PortConfiguration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let queues_str_vec: Vec<_> = self.queues.iter().map(|q| q.to_string()).collect();
        let queue_str = queues_str_vec.join(" ");
        write!(f,
               "Port {} Queue_Count: {} Queues: [ {} ] RXD: {} TXD: {} Loopback {}",
               self.name,
               self.queues.len(),
               queue_str,
               self.rxd,
               self.txd,
               self.loopback)
    }
}
