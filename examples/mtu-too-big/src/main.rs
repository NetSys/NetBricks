#![feature(box_syntax)]
#![feature(asm)]
#[macro_use]
extern crate netbricks;
use netbricks::common::Result;
use netbricks::config::{basic_opts, read_matches};
use netbricks::interface::*;
use netbricks::operators::{Batch, ReceiveBatch};
use netbricks::packets::icmp::v6::{Icmpv6, PacketTooBig};
use netbricks::packets::ip::v6::{Ipv6, IPV6_MIN_MTU};
use netbricks::packets::ip::ProtocolNumbers;
use netbricks::packets::{Ethernet, EthernetHeader, Fixed, Packet};
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
                .map(|p| p.parse::<Ethernet>())
                .group_by(
                    |eth| eth.len() > IPV6_MIN_MTU + EthernetHeader::size(),
                    |groups| {
                        compose! {
                            groups,
                            true => |group| {
                                group.map(reject_too_big)
                            },
                            false => |group| {
                                group
                            }
                        }
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

fn reject_too_big(mut ethernet: Ethernet) -> Result<Ethernet> {
    ethernet.swap_addresses();

    let mut ipv6 = ethernet.parse::<Ipv6>()?;
    let src = ipv6.src();
    let dst = ipv6.dst();
    ipv6.set_src(dst);
    ipv6.set_dst(src);
    ipv6.set_next_header(ProtocolNumbers::Icmpv6);

    let mut too_big = ipv6.push::<Icmpv6<Ipv6, PacketTooBig>>()?;
    too_big.set_mtu(IPV6_MIN_MTU as u32);
    too_big.cascade();

    Ok(too_big.deparse().deparse())
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
