#![feature(box_syntax)]
#![feature(asm)]
extern crate e2d2;
extern crate fnv;
extern crate time;
extern crate simd;
extern crate getopts;
extern crate rand;
use e2d2::interface::*;
use e2d2::interface::dpdk::*;
use e2d2::operators::*;
use e2d2::scheduler::*;
use e2d2::config::*;
use getopts::Options;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::time::Duration;
use std::thread;
use std::process;
use std::sync::Arc;
use self::nf::*;
mod nf;

const CONVERSION_FACTOR: f64 = 1000000000.;

fn test(ports: Vec<PortQueue>, sched: &mut Scheduler, delay_arg: u64) {
    for port in &ports {
        println!("Receiving port {} rxq {} txq {} w/ delay {}",
                 port.port.mac_address(),
                 port.rxq(),
                 port.txq(),
                 delay_arg);
    }

    let pipelines: Vec<_> = ports.iter()
        .map(|port| delay(ReceiveBatch::new(port.clone()), delay_arg).send(port.clone()))
        .collect();
    println!("Running {} pipelines", pipelines.len());
    for pipeline in pipelines {
        sched.add_task(pipeline);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("", "secondary", "run as a secondary process");
    opts.optflag("", "primary", "run as a primary process");
    opts.optopt("n", "name", "name to use for the current process", "name");
    opts.optmulti("p", "port", "Port to use", "[type:]id");
    opts.optmulti("c", "core", "Core to use", "core");
    opts.optopt("m", "master", "Master core", "master");
    opts.optopt("d", "delay", "Delay cycles", "cycles");
    opts.optopt("f", "configuration", "Configuration file", "path");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };
    if matches.opt_present("h") {
        print!("{}", opts.usage(&format!("Usage: {} [options]", program)));
        process::exit(0)
    }

    let delay_arg = matches.opt_str("d")
        .unwrap_or_else(|| String::from("100"))
        .parse()
        .expect("Could not parse delay");

    let configuration = if matches.opt_present("f") {
        let config_file = matches.opt_str("f").unwrap();
        match read_configuration(&config_file[..]) {
            Ok(cfg) => cfg,
            Err(e) => panic!("Could not parse configuration file {}\n {}", config_file, e.description()),
        }
    } else {
        let name = matches.opt_str("n").unwrap_or_else(|| String::from("recv"));
        SchedulerConfiguration::new_with_name(&name[..])
    };

    let configuration = if matches.opt_present("m") {
        SchedulerConfiguration {
            primary_core: matches.opt_str("m")
                .unwrap()
                .parse()
                .expect("Could not parse master core"),
            ..configuration
        }
    } else {
        configuration
    };

    let configuration = if matches.opt_present("secondary") {
        SchedulerConfiguration { secondary: true, ..configuration }
    } else {
        configuration
    };

    let configuration = if matches.opt_present("primary") {
        SchedulerConfiguration { secondary: false, ..configuration }
    } else {
        configuration
    };

    fn extract_cores_for_port(ports: &[String], cores: &[i32]) -> HashMap<String, Vec<i32>> {
        let mut cores_for_port = HashMap::<String, Vec<i32>>::new();
        for (port, core) in ports.iter().zip(cores.iter()) {
            cores_for_port.entry(port.clone()).or_insert(vec![]).push(*core)
        }
        cores_for_port
    }

    let configuration = if matches.opt_present("c") {

        let cores_str = matches.opt_strs("c");

        let cores: Vec<i32> = cores_str.iter()
            .map(|n: &String| n.parse().ok().expect(&format!("Core cannot be parsed {}", n)))
            .collect();


        let cores_for_port = extract_cores_for_port(&matches.opt_strs("p"), &cores);

        let ports_to_activate: Vec<_> = cores_for_port.keys().collect();

        let mut ports = Vec::with_capacity(ports_to_activate.len());

        for port in &ports_to_activate {
            let cores = cores_for_port.get(*port).unwrap();
            ports.push(PortConfiguration::new_with_queues(*port, cores, cores))
        }
        SchedulerConfiguration { ports: ports, ..configuration }
    } else {
        configuration
    };

    println!("Going to start with configuration {}", configuration);

    let mut config = initialize_system(&configuration).unwrap();
    config.start_schedulers();

    let delay:u64 = delay_arg;
    config.add_pipeline_to_run(Arc::new(move |p, s: &mut Scheduler| test(p, s, delay)));
    config.execute();

    let mut pkts_so_far = (0, 0);
    let mut last_printed = 0.;
    const MAX_PRINT_INTERVAL: f64 = 30.;
    const PRINT_DELAY: f64 = 15.;
    let sleep_delay = (PRINT_DELAY / 2.) as u64;
    let mut start = time::precise_time_ns() as f64 / CONVERSION_FACTOR;
    let sleep_time = Duration::from_millis(sleep_delay);
    println!("0 OVERALL RX 0.00 TX 0.00 CYCLE_PER_DELAY 0 0 0");
    loop {
        thread::sleep(sleep_time); // Sleep for a bit
        let now = time::precise_time_ns() as f64 / CONVERSION_FACTOR;
        if now - start > PRINT_DELAY {
            let mut rx = 0;
            let mut tx = 0;
            for port in config.ports.values() {
                for q in 0..port.rxqs() {
                    let (rp, tp) = port.stats(q);
                    rx += rp;
                    tx += tp;
                }
            }
            let pkts = (rx, tx);
            let rx_pkts = pkts.0 - pkts_so_far.0;
            if rx_pkts > 0 || now - last_printed > MAX_PRINT_INTERVAL {
                println!("{:.2} OVERALL RX {:.2} TX {:.2}",
                         now - start,
                         rx_pkts as f64 / (now - start),
                         (pkts.1 - pkts_so_far.1) as f64 / (now - start));
                last_printed = now;
                start = now;
                pkts_so_far = pkts;
            }
        }
    }
}
