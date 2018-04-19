extern crate getopts;
use self::getopts::{Matches, Options};
use super::{read_configuration, NetbricksConfiguration, PortConfiguration};
use common::print_error;
use std::collections::HashMap;
use std::env;
use std::process;

/// Return a `getopts::Options` struct, preset so that it's ready to parse the
/// configuration flags commonly used during Netbricks examples.
pub fn basic_opts() -> Options {
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("", "secondary", "run as a secondary process");
    opts.optflag("", "primary", "run as a primary process");
    opts.optopt("n", "name", "name to use for the current process", "name");
    opts.optmulti("p", "port", "Port to use", "[type:]id");
    opts.optmulti("c", "core", "Core to use", "core");
    opts.optopt("m", "master", "Master core", "master");
    opts.optopt("", "pool_size", "Mempool Size", "size");
    opts.optopt("", "cache_size", "Core Cache Size", "size");
    opts.optopt("f", "configuration", "Configuration file", "path");
    opts.optmulti("", "dpdk_args", "DPDK arguments", "DPDK arguments");

    opts
}

/// Read the commonly used configuration flags parsed by `basic_opts()` into
/// a `NetbricksConfiguration`. Some flags may cause side effects -- for example, the
/// help flag will print usage information and then exit the process.
pub fn read_matches(matches: &Matches, opts: &Options) -> NetbricksConfiguration {
    if matches.opt_present("h") {
        let program = env::args().next().unwrap();
        print!("{}", opts.usage(&format!("Usage: {} [options]", program)));
        process::exit(0)
    }

    if matches.opt_present("dpdk_args") {
        print!("dpdk_args: {}", matches.opt_strs("dpdk_args").join(" "));
        process::exit(0)
    };

    let configuration = if matches.opt_present("f") {
        let config_file = matches.opt_str("f").unwrap();
        match read_configuration(&config_file[..]) {
            Ok(cfg) => cfg,
            Err(ref e) => {
                print_error(e);
                process::exit(1);
            }
        }
    } else {
        let name = matches.opt_str("n").unwrap_or_else(|| String::from("recv"));
        NetbricksConfiguration::new_with_name(&name[..])
    };

    let configuration = if matches.opt_present("m") {
        NetbricksConfiguration {
            primary_core: matches
                .opt_str("m")
                .unwrap()
                .parse()
                .expect("Could not parse master core"),
            strict: true,
            ..configuration
        }
    } else {
        configuration
    };

    let configuration = if matches.opt_present("secondary") {
        NetbricksConfiguration {
            secondary: true,
            ..configuration
        }
    } else {
        configuration
    };

    let configuration = if matches.opt_present("primary") {
        NetbricksConfiguration {
            secondary: false,
            ..configuration
        }
    } else {
        configuration
    };

    let configuration = if matches.opt_present("pool_size") {
        NetbricksConfiguration {
            pool_size: matches
                .opt_str("pool_size")
                .unwrap()
                .parse()
                .expect("Could not parse mempool size"),
            ..configuration
        }
    } else {
        configuration
    };

    let configuration = if matches.opt_present("cache_size") {
        NetbricksConfiguration {
            cache_size: matches
                .opt_str("cache_size")
                .unwrap()
                .parse()
                .expect("Could not parse core cache size"),
            ..configuration
        }
    } else {
        configuration
    };

    let configuration = if matches.opt_present("c") {
        let cores_str = matches.opt_strs("c");

        let mut cores: Vec<i32> = cores_str
            .iter()
            .map(|n: &String| {
                n.parse()
                    .ok()
                    .expect(&format!("Core cannot be parsed {}", n))
            })
            .collect();

        let cores_for_port = extract_cores_for_port(&matches.opt_strs("p"), &cores);

        let ports_to_activate: Vec<_> = cores_for_port.keys().collect();

        let mut ports = Vec::with_capacity(ports_to_activate.len());

        for port in &ports_to_activate {
            let cores = cores_for_port.get(*port).unwrap();
            ports.push(PortConfiguration::new_with_queues(*port, cores, cores))
        }
        cores.dedup();
        NetbricksConfiguration {
            cores: cores,
            ports: ports,
            ..configuration
        }
    } else {
        configuration
    };

    println!("Going to start with configuration {}", configuration);
    configuration
}

fn extract_cores_for_port(ports: &[String], cores: &[i32]) -> HashMap<String, Vec<i32>> {
    let mut cores_for_port = HashMap::<String, Vec<i32>>::new();
    for (port, core) in ports.iter().zip(cores.iter()) {
        cores_for_port
            .entry(port.clone())
            .or_insert(vec![])
            .push(*core)
    }
    cores_for_port
}
