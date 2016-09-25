use toml::*;
use std::fs::File;
use std::io::Read;
use super::{ConfigurationError, ConfigurationResult, PortConfiguration, SchedulerConfiguration};
use std::convert::From;
use std::error;

/// Default configuration values
pub const DEFAULT_POOL_SIZE: u32 = 2048 - 1;
pub const DEFAULT_CACHE_SIZE: u32 = 32;
pub const DEFAULT_SECONDARY: bool = false;
pub const DEFAULT_PRIMARY_CORE: i32 = 0;
pub const DEFAULT_NAME: &'static str = "zcsi";
pub const NUM_RXD: i32 = 128;
pub const NUM_TXD: i32 = 128;

/// Read a TOML stub and figure out the port.
fn read_port(value: &Value) -> ConfigurationResult<PortConfiguration> {
    if let &Value::Table(ref port_def) = value {
        let name = match port_def.get("name") {
            Some(&Value::String(ref name)) => name.clone(),
            _ => return Err(ConfigurationError::from("Could not parse name for port")),
        };

        let rxd = match port_def.get("rxd") {
            Some(&Value::Integer(rxd)) => rxd as i32,
            None => NUM_RXD,
            v => return Err(ConfigurationError::from(format!("Could not parse number of rx descriptors {:?}", v))),
        };

        let txd = match port_def.get("txd") {
            Some(&Value::Integer(txd)) => txd as i32,
            None => NUM_TXD,
            v => return Err(ConfigurationError::from(format!("Could not parse number of tx descriptors {:?}", v))),
        };

        let loopback = match port_def.get("loopback") {
            Some(&Value::Boolean(l)) => l,
            None => false,
            v => return Err(ConfigurationError::from(format!("Could not parse loopback spec {:?}", v))),
        };

        let tso = match port_def.get("tso") {
            Some(&Value::Boolean(l)) => l,
            None => false,
            v => return Err(ConfigurationError::from(format!("Could not parse tso spec {:?}", v))),
        };

        let csum = match port_def.get("checksum") {
            Some(&Value::Boolean(l)) => l,
            None => false,
            v => return Err(ConfigurationError::from(format!("Could not parse csum spec {:?}", v))),
        };

        let symmetric_queue = port_def.contains_key("cores");
        if symmetric_queue && (port_def.contains_key("rx_cores") || port_def.contains_key("tx_cores")) {
            println!("cores specified along with rx_cores and/or tx_cores for port {}",
                     name);
            return Err(ConfigurationError::from(format!("cores specified along with rx_cores and/or tx_cores for \
                                                         port {}",
                                                        name)));
        }

        fn read_queue(queue: &Value) -> ConfigurationResult<Vec<i32>> {
            match queue {
                &Value::Array(ref queues) => {
                    let mut qs = Vec::with_capacity(queues.len());
                    for q in queues {
                        if let &Value::Integer(core) = q {
                            qs.push(core as i32)
                        } else {
                            return Err(ConfigurationError::from(format!("Could not parse queue spec {:?}", q)));
                        };
                    }
                    Ok(qs)
                }
                &Value::Integer(core) => Ok(vec![core as i32]),
                _ => Ok(vec![]),
            }
        }

        let rx_queues = if symmetric_queue {
            try!(read_queue(port_def.get("cores").unwrap()))
        } else {
            match port_def.get("rx_cores") {
                Some(v) => try!(read_queue(v)),
                None => Vec::with_capacity(0),
            }
        };

        let tx_queues = if symmetric_queue {
            rx_queues.clone()
        } else {
            match port_def.get("tx_cores") {
                Some(v) => try!(read_queue(v)),
                None => Vec::with_capacity(0),
            }
        };

        Ok(PortConfiguration {
            name: name,
            rx_queues: rx_queues,
            tx_queues: tx_queues,
            rxd: rxd,
            txd: txd,
            loopback: loopback,
            csum: csum,
            tso: tso,
        })
    } else {
        Err(ConfigurationError::from("Could not understand port spec"))
    }
}

/// Read a TOML string and create a `SchedulerConfiguration` structure.
/// `configuration` is a TOML formatted string.
/// `filename` is used for error reporting purposes, and is otherwise meaningless.
pub fn read_configuration_from_str(configuration: &str, filename: &str) -> ConfigurationResult<SchedulerConfiguration> {
    // Parse string for TOML file.
    let mut parser = Parser::new(configuration);
    let toml = match parser.parse() {
        Some(toml) => toml,
        None => {
            for err in &parser.errors {
                // FIXME: Change to logging
                let (loline, locol) = parser.to_linecol(err.lo);
                let (hiline, hicol) = parser.to_linecol(err.hi);
                println!("Parse error error: {} file {} location {}:{} -- {}:{}",
                         err.desc,
                         filename,
                         loline,
                         locol,
                         hicol,
                         hiline);
            }
            return Err(ConfigurationError::from(format!("Experienced {} parse errors in spec.", parser.errors.len())));
        }
    };

    // Get primary core from configuration.
    let master_lcore = match toml.get("master_core") {
        Some(&Value::Integer(core)) => core as i32,
        Some(&Value::String(ref core)) => {
            match core.parse() {
                Ok(c) => c,
                _ => return Err(ConfigurationError::from(format!("Could not parse {} as core", core))),
            }
        }
        None => DEFAULT_PRIMARY_CORE,
        v => {
            println!("Could not parse core");
            return Err(ConfigurationError::from(format!("Could not parse {:?} as core", v)));
        }
    };

    // Get name from configuration
    let name = match toml.get("name") {
        Some(&Value::String(ref name)) => name.clone(),
        None => String::from(DEFAULT_NAME),
        _ => {
            println!("Could not parse name");
            return Err(ConfigurationError::from("Could not parse name"));
        }
    };

    // Parse mempool size
    let pool_size = match toml.get("pool_size") {
        Some(&Value::Integer(pool)) => pool as u32,
        None => DEFAULT_POOL_SIZE,
        _ => {
            println!("Could parse pool size");
            return Err(ConfigurationError::from("Could not parse pool size"));
        }
    };

    let cache_size = match toml.get("cache_size") {
        Some(&Value::Integer(cache)) => cache as u32,
        None => DEFAULT_CACHE_SIZE,
        _ => {
            println!("Could parse cache size");
            return Err(ConfigurationError::from("Could not parse cache size"));
        }
    };

    let secondary = match toml.get("secondary") {
        Some(&Value::Boolean(secondary)) => secondary,
        None => DEFAULT_SECONDARY,
        _ => {
            println!("Could not parse whether this is a secondary process");
            return Err(ConfigurationError::from("Could not parse secondary processor spec"));
        }
    };

    let ports = match toml.get("ports") {
        Some(&Value::Array(ref ports)) => {
            let mut pouts = Vec::with_capacity(ports.len());
            for port in ports {
                let p = try!(read_port(port));
                pouts.push(p);
                // match read_port(port) {
            }
            pouts
        }
        None => Vec::with_capacity(0),
        _ => {
            println!("Ports is not an array");
            return Err(ConfigurationError::from("Ports is not an array"));
        }
    };

    Ok(SchedulerConfiguration {
        name: name,
        primary_core: master_lcore,
        secondary: secondary,
        pool_size: pool_size,
        cache_size: cache_size,
        ports: ports,
    })
}

/// Read a configuration file and create a `SchedulerConfiguration` structure.
/// `filename` should be TOML formatted file.
pub fn read_configuration(filename: &str) -> ConfigurationResult<SchedulerConfiguration> {
    let mut toml_str = String::new();
    match File::open(filename).and_then(|mut f| f.read_to_string(&mut toml_str)) {
        Ok(_) => read_configuration_from_str(&toml_str[..], filename),
        // Conflict with Error in `TOML` bah.
        Err(e) => Err(ConfigurationError::from(error::Error::description(&e))),
    }
}
