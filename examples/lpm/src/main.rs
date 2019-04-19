#![feature(box_syntax)]
#[macro_use]
extern crate netbricks;
#[macro_use]
extern crate lazy_static;
extern crate colored;
extern crate fnv;
extern crate getopts;
extern crate rand;
extern crate time;
use self::lpm::*;
use colored::*;
use netbricks::config::{basic_opts, read_matches};
use netbricks::interface::*;
use netbricks::operators::{Batch, ReceiveBatch};
use netbricks::scheduler::*;
use rand::distributions::Uniform;
use rand::{thread_rng, Rng};
use std::env;
use std::fmt::Display;
use std::net::Ipv4Addr;
use std::process;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;
use std::time::Duration;
mod lpm;

const CONVERSION_FACTOR: f64 = 1000000000.;

lazy_static! {
    static ref LOOKUP_TABLE: Arc<RwLock<IPLookup>> = {
        let mut rng = thread_rng();
        let mut lpm_table = IPLookup::new();

        for _ in 1..100 {
            let a: u8 = rng.sample(Uniform::new_inclusive(0, 255));
            let b: u8 = rng.sample(Uniform::new_inclusive(0, 255));
            let c: u8 = rng.sample(Uniform::new_inclusive(0, 255));
            let d: u8 = rng.sample(Uniform::new_inclusive(0, 255));
            lpm_table.insert_ipv4(&Ipv4Addr::new(a, b, c, d), 32, 1);
        }

        lpm_table.construct_table();
        Arc::new(RwLock::new(lpm_table))
    };
}

fn test<T, S>(ports: Vec<T>, sched: &mut S)
where
    T: PacketRx + PacketTx + Display + Clone + 'static,
    S: Scheduler + Sized,
{
    println!("Receiving started");
    for port in &ports {
        println!("Receiving port {}", port);
    }

    let pipelines: Vec<_> = ports
        .iter()
        .map(|port| {
            ReceiveBatch::new(port.clone())
                .map(|p| lpm(p))
                .group_by(
                    |v4| LOOKUP_TABLE.read().unwrap().lookup_entry(v4.src()) as usize,
                    |groups| {
                        compose!(groups,
                                 0 => |group| {
                                     group.for_each(|p| {
                                         let info_fmt = format!("{}", p.src()).magenta().bold();
                                         println!("{}", info_fmt);
                                         Ok(())
                                     })
                                 },
                                 1 => |group| {
                                     group.for_each(|p| {
                                         let info_fmt = format!("{}", p.src()).red().bold();
                                         println!("{}", info_fmt);
                                         Ok(())
                                     })
                                 },
                                 2 => |group| {
                                     group.for_each(|p| {
                                         let info_fmt = format!("{}", p.src()).blue().bold();
                                         println!("{}", info_fmt);
                                         Ok(())
                                     })
                                 }
                        );
                    },
                )
                .send(port.clone())
        })
        .collect();

    println!("Running {} pipelines", pipelines.len());
    for pipeline in pipelines {
        sched.add_task(pipeline).unwrap();
    }
}

fn main() {
    let mut opts = basic_opts();
    opts.optflag("t", "test", "Test mode do not use real ports");

    let args: Vec<String> = env::args().collect();
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };
    let configuration = read_matches(&matches, &opts);
    let phy_ports = !matches.opt_present("test");

    match initialize_system(&configuration) {
        Ok(mut context) => {
            context.start_schedulers();

            if phy_ports {
                context.add_pipeline_to_run(Arc::new(move |p, s: &mut StandaloneScheduler| {
                    test(p, s)
                }));
            } else {
                context
                    .add_test_pipeline(Arc::new(move |p, s: &mut StandaloneScheduler| test(p, s)));
            }
            context.execute();

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
                    if phy_ports {
                        for port in context.ports.values() {
                            for q in 0..port.rxqs() {
                                let (rp, tp) = port.stats(q);
                                rx += rp;
                                tx += tp;
                            }
                        }
                    } else {
                        for port in context.virtual_ports.values() {
                            let (rp, tp) = port.stats();
                            rx += rp;
                            tx += tp;
                        }
                    }
                    let pkts = (rx, tx);
                    let rx_pkts = pkts.0 - pkts_so_far.0;
                    if rx_pkts > 0 || now - last_printed > MAX_PRINT_INTERVAL {
                        println!(
                            "{:.2} OVERALL RX {:.2} TX {:.2}",
                            now - start,
                            rx_pkts as f64 / (now - start),
                            (pkts.1 - pkts_so_far.1) as f64 / (now - start)
                        );
                        last_printed = now;
                        start = now;
                        pkts_so_far = pkts;
                    }
                }
            }
        }
        Err(ref e) => {
            println!("Error: {:?}", e);
            process::exit(1);
        }
    }
}
