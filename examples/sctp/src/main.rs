extern crate netbricks;
extern crate nix;
extern crate sctp;

use self::control::*;
use netbricks::common::Result;
use netbricks::config::load_config;
use netbricks::control::sctp::*;
use netbricks::interface::{PacketRx, PacketTx};
use netbricks::operators::{Batch, ReceiveBatch};
use netbricks::packets::{Ethernet, Packet, RawPacket};
use netbricks::runtime::Runtime;
use netbricks::scheduler::Scheduler;
use std::fmt::Display;
use std::net::*;
use std::str::FromStr;
mod control;

fn install<T, S>(ports: Vec<T>, sched: &mut S)
where
    T: PacketRx + PacketTx + Display + Clone + 'static,
    S: Scheduler + Sized,
{
    let pipelines: Vec<_> = ports
        .iter()
        .map(|port| ReceiveBatch::new(port.clone()).map(swap).send(port.clone()))
        .collect();

    println!("Running {} pipelines", pipelines.len());

    for pipeline in pipelines {
        sched.add_task(pipeline).unwrap();
    }

    let addr = SocketAddrV4::new(Ipv4Addr::from_str("0.0.0.0").unwrap(), 8001);
    let control = SctpControlServer::<ControlListener>::new_streaming(SocketAddr::V4(addr));
    sched.add_task(control).unwrap();
}

fn swap(packet: RawPacket) -> Result<Ethernet> {
    let mut ethernet = packet.parse::<Ethernet>()?;
    ethernet.swap_addresses();
    Ok(ethernet)
}

fn main() -> Result<()> {
    let configuration = load_config()?;
    println!("{}", configuration);
    let mut runtime = Runtime::init(&configuration)?;
    runtime.add_pipeline_to_run(install);
    runtime.execute()
}
