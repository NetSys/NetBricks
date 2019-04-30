extern crate getopts;
extern crate netbricks;
use getopts::Options;
use netbricks::common::Result;
use netbricks::config::load_config;
use netbricks::interface::{PacketRx, PacketTx};
use netbricks::operators::{Batch, ReceiveBatch};
use netbricks::packets::ip::v4::Ipv4;
use netbricks::packets::{Ethernet, Packet, RawPacket};
use netbricks::runtime::Runtime;
use netbricks::scheduler::Scheduler;
use std::env;
use std::fmt::Display;

fn install<T, S>(ports: Vec<T>, sched: &mut S, chain_len: u32, chain_pos: u32)
where
    T: PacketRx + PacketTx + Display + Clone + 'static,
    S: Scheduler + Sized,
{
    println!("Receiving started");

    for port in &ports {
        println!(
            "Receiving port {} on chain len {} pos {}",
            port, chain_len, chain_pos
        );
    }

    let pipelines: Vec<_> = ports
        .iter()
        .map(|port| {
            ReceiveBatch::new(port.clone())
                .filter_map(move |p| chain(p, chain_len, chain_pos))
                .send(port.clone())
        })
        .collect();

    println!("Running {} pipelines", pipelines.len());

    for pipeline in pipelines {
        sched.add_task(pipeline).unwrap();
    }
}

#[inline]
pub fn chain_nf(packet: RawPacket) -> Result<RawPacket> {
    let mut ethernet = packet.parse::<Ethernet>()?;
    ethernet.swap_addresses();
    let mut ipv4 = ethernet.parse::<Ipv4>()?;
    let ttl = ipv4.ttl();
    ipv4.set_ttl(ttl - 1);
    Ok(ipv4.deparse().deparse())
}

#[inline]
pub fn chain(packet: RawPacket, len: u32, pos: u32) -> Result<Option<Ethernet>> {
    let mut chained = chain_nf(packet)?;

    for _ in 1..len {
        chained = chain_nf(chained)?;
    }

    let chained_eth = chained.parse::<Ethernet>()?;
    let chained_ipv4 = chained_eth.parse::<Ipv4>()?;

    if chained_ipv4.ttl() != 0 {
        Ok(None)
    } else {
        let mut chained_eth = chained_ipv4.deparse();

        if len % 2 == 0 || pos % 2 == 1 {
            chained_eth.swap_addresses();
            Ok(Some(chained_eth))
        } else {
            Ok(Some(chained_eth))
        }
    }
}

fn extra_opts() -> (u32, u32) {
    let mut opts = Options::new();
    opts.optopt("l", "chain", "Chain length", "length");
    opts.optopt(
        "j",
        "position",
        "Chain position (when externally chained)",
        "position",
    );
    let args: Vec<String> = env::args().collect();
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };

    let chain_len = matches
        .opt_str("l")
        .unwrap_or_else(|| String::from("1"))
        .parse()
        .expect("Could not parse chain length");

    let chain_pos = matches
        .opt_str("j")
        .unwrap_or_else(|| String::from("0"))
        .parse()
        .expect("Could not parse chain position");

    (chain_len, chain_pos)
}

fn main() -> Result<()> {
    let (chain_len, chain_pos) = extra_opts();
    let configuration = load_config()?;
    println!("{}", configuration);
    let mut runtime = Runtime::init(&configuration)?;
    runtime.add_pipeline_to_run(move |p, s| install(p, s, chain_len, chain_pos));
    runtime.execute()
}
