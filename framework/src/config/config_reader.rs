use super::{NetbricksConfiguration, PortConfiguration};
use common::*;
use std::fs::File;
use std::io::Read;
use toml::{self, Value};

/// Default configuration values
pub const DEFAULT_POOL_SIZE: u32 = 2048 - 1;
pub const DEFAULT_CACHE_SIZE: u32 = 32;
pub const DEFAULT_PRIMARY_CORE: i32 = 0;
pub const DEFAULT_NAME: &'static str = "zcsi";
pub const NUM_RXD: i32 = 128;
pub const NUM_TXD: i32 = 128;

/// Read a TOML stub and figure out the port.
fn read_port(value: &Value) -> Result<PortConfiguration> {
    if let Value::Table(ref port_def) = *value {
        let name = match port_def.get("name") {
            Some(&Value::String(ref name)) => name.clone(),
            _ => return Err(ErrorKind::ConfigurationError(String::from("Could not parse name for port")).into()),
        };

        let rxd = match port_def.get("rxd") {
            Some(&Value::Integer(rxd)) => rxd as i32,
            None => NUM_RXD,
            v => {
                return Err(
                    ErrorKind::ConfigurationError(format!("Could not parse number of rx descriptors {:?}", v)).into(),
                )
            }
        };

        let txd = match port_def.get("txd") {
            Some(&Value::Integer(txd)) => txd as i32,
            None => NUM_TXD,
            v => {
                return Err(
                    ErrorKind::ConfigurationError(format!("Could not parse number of tx descriptors {:?}", v)).into(),
                )
            }
        };

        let loopback = match port_def.get("loopback") {
            Some(&Value::Boolean(l)) => l,
            None => false,
            v => return Err(ErrorKind::ConfigurationError(format!("Could not parse loopback spec {:?}", v)).into()),
        };

        let tso = match port_def.get("tso") {
            Some(&Value::Boolean(l)) => l,
            None => false,
            v => return Err(ErrorKind::ConfigurationError(format!("Could not parse tso spec {:?}", v)).into()),
        };

        let csum = match port_def.get("checksum") {
            Some(&Value::Boolean(l)) => l,
            None => false,
            v => return Err(ErrorKind::ConfigurationError(format!("Could not parse csum spec {:?}", v)).into()),
        };

        let symmetric_queue = port_def.contains_key("cores");
        if symmetric_queue && (port_def.contains_key("rx_cores") || port_def.contains_key("tx_cores")) {
            println!(
                "cores specified along with rx_cores and/or tx_cores for port {}",
                name
            );
            return Err(ErrorKind::ConfigurationError(format!(
                "cores specified along with rx_cores and/or tx_cores \
                 for port {}",
                name
            )).into());
        }

        fn read_queue(queue: &Value) -> Result<Vec<i32>> {
            match *queue {
                Value::Array(ref queues) => {
                    let mut qs = Vec::with_capacity(queues.len());
                    for q in queues {
                        if let Value::Integer(core) = *q {
                            qs.push(core as i32)
                        } else {
                            return Err(
                                ErrorKind::ConfigurationError(format!("Could not parse queue spec {:?}", q)).into(),
                            );
                        };
                    }
                    Ok(qs)
                }
                Value::Integer(core) => Ok(vec![core as i32]),
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
        Err(ErrorKind::ConfigurationError(String::from("Could not understand port spec")).into())
    }
}

/// Read a TOML string and create a `NetbricksConfiguration` structure.
/// `configuration` is a TOML formatted string.
/// `filename` is used for error reporting purposes, and is otherwise meaningless.
pub fn read_configuration_from_str(configuration: &str, filename: &str) -> Result<NetbricksConfiguration> {
    // Parse string for TOML file.
    let toml = match toml::de::from_str::<Value>(configuration) {
        Ok(toml) => toml,
        Err(error) => {
            println!("Parse error: {} in file: {}", error, filename);
            return Err(ErrorKind::ConfigurationError(format!("Experienced {} parse errors in spec.", error)).into());
        }
    };

    // Get name from configuration
    let name = match toml.get("name") {
        Some(&Value::String(ref name)) => name.clone(),
        None => String::from(DEFAULT_NAME),
        _ => {
            println!("Could not parse name");
            return Err(ErrorKind::ConfigurationError(String::from("Could not parse name")).into());
        }
    };

    // Get primary core from configuration.
    let master_lcore = match toml.get("master_core") {
        Some(&Value::Integer(core)) => core as i32,
        Some(&Value::String(ref core)) => match core.parse() {
            Ok(c) => c,
            _ => return Err(ErrorKind::ConfigurationError(format!("Could not parse {} as core", core)).into()),
        },
        None => DEFAULT_PRIMARY_CORE,
        v => {
            println!("Could not parse core");
            return Err(ErrorKind::ConfigurationError(format!("Could not parse {:?} as core", v)).into());
        }
    };

    // Parse mempool size
    let pool_size = match toml.get("pool_size") {
        Some(&Value::Integer(pool)) => pool as u32,
        None => DEFAULT_POOL_SIZE,
        _ => {
            println!("Could parse pool size");
            return Err(ErrorKind::ConfigurationError(String::from("Could not parse pool size")).into());
        }
    };

    // Get cache size
    let cache_size = match toml.get("cache_size") {
        Some(&Value::Integer(cache)) => cache as u32,
        None => DEFAULT_CACHE_SIZE,
        _ => {
            println!("Could parse cache size");
            return Err(ErrorKind::ConfigurationError(String::from("Could not parse cache size")).into());
        }
    };

    let cores = match toml.get("cores") {
        Some(&Value::Array(ref c)) => {
            let mut cores = Vec::with_capacity(c.len());
            for core in c {
                if let Value::Integer(core) = *core {
                    cores.push(core as i32)
                } else {
                    return Err(ErrorKind::ConfigurationError(format!("Could not parse core spec {}", core)).into());
                }
            }
            cores
        }
        None => Vec::with_capacity(0),
        _ => {
            println!("Cores is not an array");
            return Err(ErrorKind::ConfigurationError(String::from("Cores is not an array")).into());
        }
    };

    let strict = match toml.get("strict") {
        Some(&Value::Boolean(l)) => l,
        None => false,
        v => {
            return Err(ErrorKind::ConfigurationError(format!(
                "Could not parse strict spec (should be boolean) {:?}",
                v
            )).into())
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
            return Err(ErrorKind::ConfigurationError(String::from("Ports is not an array")).into());
        }
    };

    Ok(NetbricksConfiguration {
        name: name,
        primary_core: master_lcore,
        cores: cores,
        strict: strict,
        pool_size: pool_size,
        cache_size: cache_size,
        ports: ports,
        dpdk_args: None,
    })
}

/// Read a configuration file and create a `NetbricksConfiguration` structure.
/// `filename` should be TOML formatted file.
pub fn read_configuration(filename: &str) -> Result<NetbricksConfiguration> {
    let mut toml_str = String::new();
    let _ = try!{File::open(filename).and_then(|mut f| f.read_to_string(&mut toml_str))
    .chain_err(|| ErrorKind::ConfigurationError(String::from("Could not read file")))};
    read_configuration_from_str(&toml_str[..], filename)
}
