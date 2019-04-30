use clap::ArgMatches;
use config_rs::{Config, ConfigError, File, FileFormat, Source, Value};
use std::collections::HashMap;
use std::fmt;

pub const DEFAULT_POOL_SIZE: u32 = 2048 - 1;
pub const DEFAULT_CACHE_SIZE: u32 = 32;
pub const NUM_RXD: i32 = 128;
pub const NUM_TXD: i32 = 128;

/// NetBricks configuration
#[derive(Debug, Default, Deserialize)]
pub struct NetBricksConfiguration {
    /// Name, this is passed on to DPDK. If you want to run multiple DPDK apps,
    /// this needs to be unique per application.
    pub name: String,
    /// Should this process be run as a secondary process or a primary process?
    pub secondary: bool,
    /// Where should the main thread (for the examples this just sits around and
    /// prints packet counts) be run.
    pub primary_core: i32,
    /// Cores that can be used by NetBricks. Note that currently we will add any
    /// cores specified in the ports configuration to this list, unless told not
    /// to using the next option.
    pub cores: Vec<i32>,
    /// Use the core list as a strict list, i.e., error out if any cores with an
    /// rxq or txq are not specified on the core list. This is set to false by
    /// default because of laziness.
    pub strict: bool,
    /// A set of ports to be initialized.
    pub ports: Vec<PortConfiguration>,
    /// Memory pool size: sizing this pool is a bit complex; too big and you might
    /// affect caching behavior, too small and you limit how many packets are in
    /// your system overall.
    pub pool_size: u32,
    /// Size of the per-core mempool cache.
    pub cache_size: u32,
    /// Custom DPDK arguments.
    pub dpdk_args: Option<String>,
}

impl fmt::Display for NetBricksConfiguration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ports = self
            .ports
            .iter()
            .map(|p| format!("\t{}", p))
            .collect::<Vec<_>>()
            .join("\n");

        write!(
            f,
            "name: {}, secondary: {}, pool size: {}, cache size: {}\nprimary core: {}, cores: {:?}, strict: {}\nports:\n{}\nDPDK args: {:?}",
            self.name,
            self.secondary,
            self.pool_size,
            self.cache_size,
            self.primary_core,
            self.cores,
            self.strict,
            ports,
            self.dpdk_args,
        )
    }
}

/// Port (network device) configuration
#[derive(Debug, Default, Deserialize)]
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

impl fmt::Display for PortConfiguration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "name: {}, rxq: {:?}, txq: {:?}, rxd: {}, txd: {}, loopback: {}, tso: {}, csum: {}",
            self.name,
            self.rx_queues,
            self.tx_queues,
            self.rxd,
            self.txd,
            self.loopback,
            self.tso,
            self.csum,
        )
    }
}

lazy_static! {
    pub(crate) static ref CLI_ARGS: ArgMatches<'static> = clap_app!(app =>
        (version: "0.3.0")
        (@arg file: -f --file +takes_value "custom configuration file")
        (@arg name: -n --name +takes_value "DPDK process name")
        (@group process_mode =>
            (@arg primary: --primary conflicts_with[secondary] "run as a primary process")
            (@arg secondary: --secondary conflicts_with[primary] "run as a secondary process")
        )
        (@arg primary_core: --("primary-core") +takes_value "the core to run the main thread on")
        (@arg cores: -c --core ... +takes_value "core that NetBricks can use")
        (@arg ports: -p --port ... +takes_value "port to be initialized")
        (@arg pool_size: --("pool-size") +takes_value "memory pool size")
        (@arg cache_size: --("cache-size") +takes_value "per core cache size")
        (@arg dpdk_args: --("dpdk-args") ... +takes_value "custom DPDK arguments")
        (@arg duration: -d --duration +takes_value "test duration")
    )
    .get_matches();
}

// this struct is used to indirectly implement `Source` for `ArgMatches` and
// allow for command line args integrated with file based configuration
#[derive(Clone, Debug)]
struct CommandLine();

impl Source for CommandLine {
    fn clone_into_box(&self) -> Box<Source + Send + Sync> {
        Box::new((*self).clone())
    }

    // TODO: allow dpdk_args to pass-through and affect EAL
    fn collect(&self) -> Result<HashMap<String, Value>, ConfigError> {
        let mut map = HashMap::new();
        let uri = "the command line".to_string();
        let uri = Some(&uri);

        if let Some(name) = CLI_ARGS.value_of("name") {
            map.insert("name".to_string(), Value::new(uri, name));
        }

        if CLI_ARGS.is_present("process_mode") {
            let secondary = CLI_ARGS.is_present("secondary");
            map.insert("secondary".to_string(), Value::new(uri, secondary));
        }

        if CLI_ARGS.is_present("primary_core") {
            let core = value_t!(CLI_ARGS, "primary_core", i32)
                .map_err(|err| ConfigError::Foreign(Box::new(err)))?;
            map.insert("primary_core".to_string(), Value::new(uri, core as i64));
        }

        if CLI_ARGS.is_present("pool_size") {
            let pool_size = value_t!(CLI_ARGS, "pool_size", u32)
                .map_err(|err| ConfigError::Foreign(Box::new(err)))?;
            map.insert("pool_size".to_string(), Value::new(uri, pool_size as i64));
        }

        if CLI_ARGS.is_present("cache_size") {
            let cache_size = value_t!(CLI_ARGS, "cache_size", u32)
                .map_err(|err| ConfigError::Foreign(Box::new(err)))?;
            map.insert("cache_size".to_string(), Value::new(uri, cache_size as i64));
        }

        if let Some(ports) = CLI_ARGS.values_of("ports") {
            let cores = values_t!(CLI_ARGS, "cores", i32)
                .map_err(|err| ConfigError::Foreign(Box::new(err)))?;
            let cores = cores.iter().map(|&core| core as i64).collect::<Vec<_>>();

            let ports = ports
                .zip(cores.iter())
                .map(|(port, &core)| {
                    [
                        ("name".to_string(), Value::new(uri, port)),
                        ("rx_queues".to_string(), Value::new(uri, vec![core])),
                        ("tx_queues".to_string(), Value::new(uri, vec![core])),
                        ("rxd".to_string(), Value::new(uri, NUM_RXD as i64)),
                        ("txd".to_string(), Value::new(uri, NUM_TXD as i64)),
                        ("loopback".to_string(), Value::new(uri, false)),
                        ("tso".to_string(), Value::new(uri, false)),
                        ("csum".to_string(), Value::new(uri, false)),
                    ]
                    .iter()
                    .cloned()
                    .collect::<HashMap<_, _>>()
                })
                .collect::<Vec<_>>();

            map.insert("ports".to_string(), Value::new(uri, ports));
            map.insert("cores".to_string(), Value::new(uri, cores));
        }

        Ok(map)
    }
}

static DEFAULT_TOML: &'static str = r#"
    name = "netbricks"
    secondary = false
    primary_core = 0
    cores = [0]
    strict = false
    pool_size = 2047
    cache_size = 32
    ports = []
    duration = 0
"#;

/// Loads the configuration
///
/// Configuration can be specified through either a file or command
/// line. Command line arguments will have precedence over settings
/// from the configuration file.
pub fn load_config() -> Result<NetBricksConfiguration, ConfigError> {
    let mut config = Config::new();
    config.merge(File::from_str(DEFAULT_TOML, FileFormat::Toml))?;

    if let Some(filename) = CLI_ARGS.value_of("file") {
        config.merge(File::with_name(filename))?;
    }

    config.merge(CommandLine())?;
    config.try_into()
}
