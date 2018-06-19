#![feature(box_syntax)]
#![feature(asm)]
#![feature(ip_constructors)]
extern crate colored;
extern crate fnv;
extern crate generic_array;
#[macro_use]
extern crate netbricks;
#[macro_use]
extern crate lazy_static;
use self::nf::*;
use fnv::FnvHasher;
use netbricks::config::{basic_opts, read_matches};
use netbricks::headers::*;
use netbricks::interface::*;
use netbricks::operators::*;
use netbricks::scheduler::*;
use std::collections::HashMap;
use std::env;
use std::fmt::Display;
use std::hash::BuildHasherDefault;
use std::process;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;
use std::time::Duration;
mod nf;

type FnvHash = BuildHasherDefault<FnvHasher>;

lazy_static! {
    static ref CACHE: RwLock<HashMap<usize, Vec<Segment>, FnvHash>> = {
        let m = HashMap::with_hasher(Default::default());
        RwLock::new(m)
    };
}

fn test<T, S>(ports: Vec<T>, sched: &mut S)
where
    T: PacketRx + PacketTx + Display + Clone + 'static,
    S: Scheduler + Sized,
{
    println!("Receiving started");

    let pipelines: Vec<_> = ports
        .iter()
        .map(|port| nf(ReceiveBatch::new(port.clone()), sched).send(port.clone()))
        .collect();
    println!("Running {} pipelines", pipelines.len());
    for pipeline in pipelines {
        sched.add_task(pipeline).unwrap();
    }
}

fn main() {
    let mut opts = basic_opts();
    opts.optopt(
        "",
        "dur",
        "Test duration",
        "If this option is set to a nonzero value, then the \
         test will just loop after 2 seconds",
    );

    let args: Vec<String> = env::args().collect();
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };

    let configuration = read_matches(&matches, &opts);

    let test_duration: u64 = matches
        .opt_str("dur")
        .unwrap_or_else(|| String::from("0"))
        .parse()
        .expect("Could not parse test duration");

    match initialize_system(&configuration) {
        Ok(mut context) => {
            context.start_schedulers();
            context.add_pipeline_to_run(Arc::new(move |p, s: &mut StandaloneScheduler| test(p, s)));
            context.execute();

            if test_duration != 0 {
                thread::sleep(Duration::from_secs(test_duration));
            } else {
                loop {
                    thread::sleep(Duration::from_secs(2));
                }
            }
        }
        Err(ref e) => {
            println!("Error: {}", e);
            if let Some(backtrace) = e.backtrace() {
                println!("Backtrace: {:?}", backtrace);
            }
            process::exit(1);
        }
    }
}
