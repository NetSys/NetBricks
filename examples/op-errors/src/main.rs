#![feature(box_syntax)]
#![feature(asm)]
#[macro_use]
extern crate log;
extern crate netbricks;
extern crate simplelog;
#[macro_use]
extern crate failure;

use log::Level;
use netbricks::common::Result;
use netbricks::config::{basic_opts, read_matches};
use netbricks::interface::*;
use netbricks::operators::{Batch, ReceiveBatch};
use netbricks::packets::ip::v6::Ipv6;
use netbricks::packets::{EtherTypes, Ethernet, Packet};
use netbricks::scheduler::*;
use simplelog::{Config as SimpleConfig, LevelFilter, WriteLogger};
use std::env;
use std::fmt::Display;
use std::fs::File as StdFile;
use std::net::Ipv6Addr;
use std::process;
use std::str::FromStr;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn start_logger() {
    WriteLogger::init(
        LevelFilter::Warn,
        SimpleConfig {
            time: None,
            level: Some(Level::Error),
            target: Some(Level::Debug),
            location: Some(Level::Trace),
            time_format: None,
        },
        StdFile::create("test.log").unwrap(),
    )
    .unwrap();
}

fn test<T, S>(ports: Vec<T>, sched: &mut S)
where
    T: PacketRx + PacketTx + Display + Clone + 'static,
    S: Scheduler + Sized,
{
    println!("Receiving started");

    let pipelines: Vec<_> = ports
        .iter()
        .map(|port| {
            ReceiveBatch::new(port.clone())
                .map(|p| p.parse::<Ethernet>())
                .filter(|p| match p.ether_type() {
                    EtherTypes::Ipv6 => true,
                    _ => false,
                })
                .map(|p| {
                    let v6 = p.parse::<Ipv6>()?;
                    throw_mama_from_the_train(v6)
                })
                .for_each(|p| {
                    warn!("v6: {}", p);
                    Ok(())
                })
                .send(port.clone())
        })
        .collect();
    println!("Running {} pipelines", pipelines.len());
    for pipeline in pipelines {
        sched.add_task(pipeline).unwrap();
    }
}

fn throw_mama_from_the_train(packet: Ipv6) -> Result<Ipv6> {
    let hextets = packet.src().segments();
    let prefix = Ipv6Addr::new(hextets[0], hextets[1], hextets[2], hextets[3], 0, 0, 0, 0);
    if prefix == Ipv6Addr::from_str("da75::").unwrap() {
        bail!("directed by danny devito")
    } else {
        Ok(packet)
    }
}

fn main() {
    start_logger();

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
            println!("Error: {:?}", e);
            process::exit(1);
        }
    }
}
