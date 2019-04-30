#[macro_use]
extern crate log;
extern crate netbricks;
extern crate simplelog;
#[macro_use]
extern crate failure;

use log::Level;
use netbricks::common::Result;
use netbricks::config::load_config;
use netbricks::interface::{PacketRx, PacketTx};
use netbricks::operators::{Batch, ReceiveBatch};
use netbricks::packets::ip::v6::Ipv6;
use netbricks::packets::{EtherTypes, Ethernet, Packet};
use netbricks::runtime::Runtime;
use netbricks::scheduler::Scheduler;
use netbricks::utils::cidr::v6::Ipv6Cidr;
use netbricks::utils::cidr::Cidr;
use simplelog::{Config as SimpleConfig, LevelFilter, WriteLogger};
use std::fmt::Display;
use std::fs::File as StdFile;
use std::str::FromStr;

static BAD_PREFIX: &str = "da75::0/16";

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

fn install<T, S>(ports: Vec<T>, sched: &mut S)
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
                .filter(|p| p.ether_type() == EtherTypes::Ipv6)
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
    let cidr = Ipv6Cidr::from_str(BAD_PREFIX).unwrap();

    if cidr.contains(packet.src()) {
        bail!("directed by danny devito")
    } else {
        Ok(packet)
    }
}

fn main() -> Result<()> {
    start_logger();
    let configuration = load_config()?;
    println!("{}", configuration);
    let mut runtime = Runtime::init(&configuration)?;
    runtime.add_pipeline_to_run(install);
    runtime.execute()
}
