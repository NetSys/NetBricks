extern crate colored;
extern crate fnv;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate netbricks;
extern crate rand;
use self::lpm::*;
use colored::*;
use netbricks::common::Result;
use netbricks::config::load_config;
use netbricks::interface::{PacketRx, PacketTx};
use netbricks::operators::{Batch, ReceiveBatch};
use netbricks::runtime::Runtime;
use netbricks::scheduler::Scheduler;
use rand::distributions::Uniform;
use rand::{thread_rng, Rng};
use std::fmt::Display;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::sync::RwLock;
mod lpm;

lazy_static! {
    static ref LOOKUP_TABLE: Arc<RwLock<IPLookup>> = {
        let mut rng = thread_rng();
        let mut lpm_table = IPLookup::new();

        for _ in 1..100 {
            let a: u8 = rng.sample(Uniform::new_inclusive(0, 255));
            let b: u8 = rng.sample(Uniform::new_inclusive(0, 255));
            let c: u8 = rng.sample(Uniform::new_inclusive(0, 255));
            let d: u8 = rng.sample(Uniform::new_inclusive(0, 255));
            lpm_table.insert_ipv4(Ipv4Addr::new(a, b, c, d), 32, 1);
        }

        lpm_table.construct_table();
        Arc::new(RwLock::new(lpm_table))
    };
}

fn install<T, S>(ports: Vec<T>, sched: &mut S)
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
                .map(lpm)
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

fn main() -> Result<()> {
    let configuration = load_config()?;
    println!("{}", configuration);
    let mut runtime = Runtime::init(&configuration)?;
    runtime.add_pipeline_to_run(install);
    runtime.execute()
}
