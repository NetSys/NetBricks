#![feature(box_syntax)]
#![feature(asm)]
extern crate colored;
#[macro_use]
extern crate netbricks;
use colored::*;
use netbricks::common::Result;
use netbricks::config::{basic_opts, read_matches};
use netbricks::interface::*;
use netbricks::operators::{Batch, ReceiveBatch};
use netbricks::packets::ip::v4::Ipv4;
use netbricks::packets::ip::v6::Ipv6;
use netbricks::packets::ip::IpPacket;
use netbricks::packets::{EtherTypes, Ethernet, Packet, RawPacket, Tcp};
use netbricks::scheduler::*;
use std::env;
use std::fmt::Display;
use std::process;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

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
                .map(eth_nf)
                .group_by(
                    |ethernet| ethernet.ether_type(),
                    |groups| {
                        compose!(
                            groups,
                            EtherTypes::Ipv4 => |group| {
                                group.map(ipv4_nf)
                            },
                            EtherTypes::Ipv6 => |group| {
                                group.map(ipv6_nf)
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

#[inline]
pub fn eth_nf(packet: RawPacket) -> Result<Ethernet> {
    let ethernet = packet.parse::<Ethernet>()?;

    let info_fmt = format!("[eth] {}", ethernet).magenta().bold();
    println!("{}", info_fmt);

    Ok(ethernet)
}

#[inline]
pub fn ipv4_nf(ethernet: Ethernet) -> Result<Ethernet> {
    let ipv4 = ethernet.parse::<Ipv4>()?;
    let info_fmt = format!("[ipv4] {}, [offset] {}", ipv4, ipv4.offset()).yellow();
    println!("{}", info_fmt);

    let tcp = ipv4.parse::<Tcp<Ipv4>>()?;
    print_tcp(&tcp);

    Ok(tcp.deparse().deparse())
}

#[inline]
pub fn ipv6_nf(ethernet: Ethernet) -> Result<Ethernet> {
    let ipv6 = ethernet.parse::<Ipv6>()?;
    let info_fmt = format!("[ipv6] {}, [offset] {}", ipv6, ipv6.offset()).cyan();
    println!("{}", info_fmt);

    let tcp = ipv6.parse::<Tcp<Ipv6>>()?;
    print_tcp(&tcp);

    Ok(tcp.deparse().deparse())
}

#[inline]
fn print_tcp<T: IpPacket>(tcp: &Tcp<T>) {
    let tcp_fmt = format!("[tcp] {}", tcp).green();
    println!("{}", tcp_fmt);

    let flow_fmt = format!("[flow] {}", tcp.flow()).bright_blue();
    println!("{}", flow_fmt);
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
            println!("Error: {:?}", e);
            process::exit(1);
        }
    }
}
