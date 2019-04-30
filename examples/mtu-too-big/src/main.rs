#[macro_use]
extern crate netbricks;
use netbricks::common::Result;
use netbricks::config::load_config;
use netbricks::interface::{PacketRx, PacketTx};
use netbricks::operators::{Batch, ReceiveBatch};
use netbricks::packets::icmp::v6::{Icmpv6, PacketTooBig};
use netbricks::packets::ip::v6::{Ipv6, IPV6_MIN_MTU};
use netbricks::packets::ip::ProtocolNumbers;
use netbricks::packets::{Ethernet, EthernetHeader, Fixed, Packet};
use netbricks::runtime::Runtime;
use netbricks::scheduler::Scheduler;
use std::fmt::Display;

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

fn main() -> Result<()> {
    let configuration = load_config()?;
    println!("{}", configuration);
    let mut runtime = Runtime::init(&configuration)?;
    runtime.add_pipeline_to_run(install);
    runtime.execute()
}
