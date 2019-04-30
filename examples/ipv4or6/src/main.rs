extern crate colored;
#[macro_use]
extern crate netbricks;
use colored::*;
use netbricks::common::Result;
use netbricks::config::load_config;
use netbricks::interface::{PacketRx, PacketTx};
use netbricks::operators::{Batch, ReceiveBatch};
use netbricks::packets::ip::v4::Ipv4;
use netbricks::packets::ip::v6::Ipv6;
use netbricks::packets::ip::IpPacket;
use netbricks::packets::{EtherTypes, Ethernet, Packet, RawPacket, Tcp};
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

fn main() -> Result<()> {
    let configuration = load_config()?;
    println!("{}", configuration);
    let mut runtime = Runtime::init(&configuration)?;
    runtime.add_pipeline_to_run(install);
    runtime.execute()
}
