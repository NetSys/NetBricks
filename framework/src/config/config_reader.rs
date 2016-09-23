use toml::*;
use std::fs::File;
use std::io::Read;
use interface::{NUM_RXD, NUM_TXD};
use super::{PortConfiguration, SchedulerConfiguration};

/// Default configuration values
pub const DEFAULT_POOL_SIZE: u32 = 2048 - 1;
pub const DEFAULT_CACHE_SIZE: u32 = 32;
pub const DEFAULT_SECONDARY: bool = false;
pub const DEFAULT_PRIMARY_CORE: i32 = 0;
pub const DEFAULT_NAME: &'static str = "zcsi";

/// Read a TOML stub and figure out the port.
fn read_port(value: &Value) -> Option<PortConfiguration> {
    if let &Value::Table(ref port_def) = value {
        let name = match port_def.get("name") {
            Some(&Value::String(ref name)) => name.clone(),
            _ => return None,
        };

        let rxd = match port_def.get("rxd") {
            Some(&Value::Integer(rxd)) => rxd as i32,
            None => NUM_RXD,
            _ => return None,
        };

        let txd = match port_def.get("txd") {
            Some(&Value::Integer(txd)) => txd as i32,
            None => NUM_TXD,
            _ => return None,
        };

        let loopback = match port_def.get("loopback") {
            Some(&Value::Boolean(l)) => l,
            None => false,
            _ => return None,
        };

        let queues = match port_def.get("cores") {
            Some(&Value::Array(ref queues)) => {
                let mut qs = Vec::with_capacity(queues.len());
                for q in queues {
                    if let &Value::Integer(core) = q {
                        qs.push(core as i32)
                    } else {
                        return None;
                    };
                }
                qs
            }
            Some(&Value::Integer(core)) => vec![core as i32],
            None => Vec::with_capacity(0), // Allow cases where no queues are initialized.
            _ => return None,
        };

        Some(PortConfiguration {
            name: name,
            queues: queues,
            rxd: rxd,
            txd: txd,
            loopback: loopback,
        })
    } else {
        None
    }
}

/// Read a TOML string and create a `SchedulerConfiguration` structure.
/// `configuration` is a TOML formatted string.
/// `filename` is used for error reporting purposes, and is otherwise meaningless.
pub fn read_configuration_from_str(configuration: &str, filename: &str) -> Option<SchedulerConfiguration> {
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
            return None;
        }
    };

    // Get primary core from configuration.
    let master_lcore = match toml.get("master_core") {
        Some(&Value::Integer(core)) => core as i32,
        Some(&Value::String(ref core)) => {
            match core.parse() {
                Ok(c) => c,
                _ => return None,
            }
        }
        None => DEFAULT_PRIMARY_CORE,
        _ => {
            println!("Could not parse core");
            return None;
        }
    };

    // Get name from configuration
    let name = match toml.get("name") {
        Some(&Value::String(ref name)) => name.clone(),
        None => String::from(DEFAULT_NAME),
        _ => {
            println!("Could not parse name");
            return None;
        }
    };

    // Parse mempool size
    let pool_size = match toml.get("pool_size") {
        Some(&Value::Integer(pool)) => pool as u32,
        None => DEFAULT_POOL_SIZE,
        _ => {
            println!("Could parse pool size");
            return None;
        }
    };

    let cache_size = match toml.get("cache_size") {
        Some(&Value::Integer(cache)) => cache as u32,
        None => DEFAULT_CACHE_SIZE,
        _ => {
            println!("Could parse cache size");
            return None;
        }
    };

    let secondary = match toml.get("secondary") {
        Some(&Value::Boolean(secondary)) => secondary,
        None => DEFAULT_SECONDARY,
        _ => {
            println!("Could not parse whether this is a secondary process");
            return None;
        }
    };

    let ports = match toml.get("ports") {
        Some(&Value::Array(ref ports)) => {
            let mut pouts = Vec::with_capacity(ports.len());
            for port in ports {
                match read_port(port) {
                    Some(p) => pouts.push(p),
                    None => {
                        println!("Could not parse port {}", port);
                        return None;
                    }
                };
            }
            pouts
        }
        None => Vec::with_capacity(0),
        _ => {
            println!("Ports is not an array");
            return None;
        }
    };

    Some(SchedulerConfiguration {
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
pub fn read_configuration(filename: &str) -> Option<SchedulerConfiguration> {
    let mut toml_str = String::new();
    match File::open(filename).and_then(|mut f| f.read_to_string(&mut toml_str)) {
        Ok(_) => read_configuration_from_str(&toml_str[..], filename),
        _ => None,
    }
}
