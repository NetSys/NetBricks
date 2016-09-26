use std::fmt;
use std::thread::{self, JoinHandle};
use std::sync::mpsc::{SyncSender, sync_channel};
use interface::{PmdPort, PortQueue};
use interface::dpdk::{init_system, init_thread};
use scheduler::*;
use std::sync::Arc;
use std::collections::HashMap;
use std::convert::From;
use std::error::Error;

pub use self::config_reader::*;
mod config_reader;

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

#[derive(Default)]
pub struct NetBricksContext {
    pub ports: HashMap<String, Arc<PmdPort>>,
    pub rx_queues: HashMap<i32, Vec<PortQueue>>,
    pub active_cores: Vec<i32>,
    pub scheduler_channels: HashMap<i32, SyncSender<SchedulerCommand>>,
    pub scheduler_handles: HashMap<i32, JoinHandle<()>>,
}

pub fn initialize_system(configuration: &SchedulerConfiguration) -> ConfigurationResult<NetBricksContext> {
    init_system(configuration);
    let mut ctx: NetBricksContext = Default::default();
    for port in &configuration.ports {
        if ctx.ports.contains_key(&port.name) {
            println!("Port {} appears twice in specification", port.name);
            return Err(ConfigurationError::from(format!("Port {} appears twice in specification", port.name)));
        } else {
            match PmdPort::new_port_from_configuration(port) {
                Ok(p) => {
                    ctx.ports.insert(port.name.clone(), p);
                }
                Err(e) => {
                    return Err(ConfigurationError::from(format!("Port {} could not be initialized {:?}", port.name, e)))
                }
            }

            let port_instance = ctx.ports.get(&port.name).unwrap();

            for (rx_q, core) in port.rx_queues.iter().enumerate() {
                let rx_q = rx_q as i32;
                match PmdPort::new_queue_pair(port_instance, rx_q, rx_q) {
                    Ok(q) => {
                        ctx.rx_queues.entry(*core).or_insert(vec![]).push(q);
                    }
                    Err(e) => {
                        return Err(ConfigurationError::from(format!("Queue {} on port {} could not be initialized \
                                                                     {:?}",
                                                                    rx_q,
                                                                    port.name,
                                                                    e)))
                    }
                }
            }
        }
    }
    ctx.active_cores = ctx.rx_queues.keys().map(|x| *x).collect();
    Ok(ctx)
}

impl NetBricksContext {
    pub fn start_schedulers(&mut self) {
        let cores = self.active_cores.clone();
        for core in &cores {
            self.start_scheduler(*core);
        }
    }

    #[inline]
    fn start_scheduler(&mut self, core: i32) {
        let builder = thread::Builder::new();
        let (sender, receiver) = sync_channel(0);
        self.scheduler_channels.insert(core, sender);
        let join_handle = builder.name(format!("sched-{}", core).into())
            .spawn(move || {
                init_thread(core, core);
                // Other init?
                let mut sched = Scheduler::new_with_channel(receiver);
                sched.handle_requests()
            })
            .unwrap();
        self.scheduler_handles.insert(core, join_handle);
    }

    pub fn add_pipeline_to_run<T: Fn(Vec<PortQueue>, &mut Scheduler) + Send + Sync + 'static>(&mut self, run: Arc<T>) {
        for (core, channel) in &self.scheduler_channels {
            let ports = match self.rx_queues.get(core) {
                Some(v) => v.clone(),
                None => vec![],
            };
            let boxed_run = run.clone();
            channel.send(SchedulerCommand::Run(Arc::new(move |s| boxed_run(ports.clone(), s)))).unwrap();
        }
    }

    pub fn execute(&mut self) {
        for (core, channel) in &self.scheduler_channels {
            channel.send(SchedulerCommand::Execute).unwrap();
            println!("Starting scheduler on {}", core);
        }
    }
}
