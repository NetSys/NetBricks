use std::fmt;
use std::convert::From;
use std::error::Error;

pub use self::config_reader::*;
pub use self::context::*;
mod config_reader;
mod context;

#[derive(Debug)]
pub struct ConfigurationError {
    pub description: String,
}

impl fmt::Display for ConfigurationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Configuration Error: {}", self.description)
    }
}

impl Error for ConfigurationError {
    fn description(&self) -> &str {
        &self.description[..]
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}


impl From<String> for ConfigurationError {
    fn from(description: String) -> ConfigurationError {
        ConfigurationError { description: description }
    }
}

impl<'a> From<&'a String> for ConfigurationError {
    fn from(description: &'a String) -> ConfigurationError {
        ConfigurationError { description: description.clone() }
    }
}

impl<'a> From<&'a str> for ConfigurationError {
    fn from(description: &'a str) -> ConfigurationError {
        ConfigurationError { description: String::from(description) }
    }
}

pub type ConfigurationResult<T> = Result<T, ConfigurationError>;

/// `NetBricks` control configuration. In theory all applications create one of these, either through the use of
/// `read_configuration` or manually using args.
pub struct NetbricksConfiguration {
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

/// Create an empty `NetbricksConfiguration`, useful when initializing through arguments.
impl Default for NetbricksConfiguration {
    fn default() -> NetbricksConfiguration {
        NetbricksConfiguration {
            name: String::new(),
            pool_size: DEFAULT_POOL_SIZE,
            cache_size: DEFAULT_CACHE_SIZE,
            primary_core: 0,
            secondary: false,
            ports: vec![],
        }
    }
}

impl NetbricksConfiguration {
    /// Create a `NetbricksConfiguration` with a name.
    pub fn new_with_name(name: &str) -> NetbricksConfiguration {
        NetbricksConfiguration { name: String::from(name), ..Default::default() }
    }
}

impl fmt::Display for NetbricksConfiguration {
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

/// Configuration for each port (network device) in `NetBricks`.
pub struct PortConfiguration {
    /// Name. The exact semantics vary by backend. For DPDK, we allow things of the form:
    ///    <PCI ID> : Hardware device with PCI ID
    ///    dpdk:<PMD Descriptor>: PMD driver with arguments
    ///    bess:<port_name>: BESS RingVport with name.
    ///    ovs:<port_id>: OVS ring with ID.
    pub name: String,
    /// Core on which receive node for a given queue lives.
    pub rx_queues: Vec<i32>,
    /// Core on which sending node lives.
    pub tx_queues: Vec<i32>,
    /// Number of RX descriptors to use.
    pub rxd: i32,
    /// Number of TX descriptors to use.
    pub txd: i32,
    pub loopback: bool,
    pub tso: bool,
    pub csum: bool,
}

impl Default for PortConfiguration {
    fn default() -> PortConfiguration {
        PortConfiguration {
            name: String::new(),
            rx_queues: vec![],
            tx_queues: vec![],
            rxd: NUM_RXD,
            txd: NUM_TXD,
            loopback: false,
            tso: false,
            csum: false,
        }
    }
}

impl PortConfiguration {
    pub fn new_with_name(name: &str) -> PortConfiguration {
        PortConfiguration { name: String::from(name), ..Default::default() }
    }

    pub fn new_with_queues(name: &str, rx_queues: &[i32], tx_queues: &[i32]) -> PortConfiguration {
        PortConfiguration {
            rx_queues: Vec::from(rx_queues),
            tx_queues: Vec::from(tx_queues),
            ..PortConfiguration::new_with_name(name)
        }
    }
}

impl fmt::Display for PortConfiguration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let rx_queues_str_vec: Vec<_> = self.rx_queues.iter().map(|q| q.to_string()).collect();
        let rx_queue_str = rx_queues_str_vec.join(" ");
        let tx_queues_str_vec: Vec<_> = self.tx_queues.iter().map(|q| q.to_string()).collect();
        let tx_queue_str = tx_queues_str_vec.join(" ");
        write!(f,
               "Port {} RXQ_Count: {} RX_Queues: [ {} ] TXQ_Count: {} TX_Queues: {} RXD: {} TXD: {} Loopback {}",
               self.name,
               self.rx_queues.len(),
               rx_queue_str,
               self.tx_queues.len(),
               tx_queue_str,
               self.rxd,
               self.txd,
               self.loopback)
    }
}
