#[macro_use]
extern crate lazy_static;
extern crate netbricks;
use netbricks::common::Result;
use netbricks::config::load_config;
use netbricks::interface::{PacketRx, PacketTx};
use netbricks::operators::{Batch, ReceiveBatch};
use netbricks::packets::ip::v4::Ipv4;
use netbricks::packets::ip::v6::Ipv6;
use netbricks::packets::ip::{ProtocolNumber, ProtocolNumbers};
use netbricks::packets::{EtherTypes, Ethernet, Packet, RawPacket};
use netbricks::runtime::Runtime;
use netbricks::scheduler::Scheduler;
use std::fmt::Display;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

struct Metrics {
    tcp: AtomicU64,
    udp: AtomicU64,
    icmp: AtomicU64,
    other: AtomicU64,
}

impl Metrics {
    fn new() -> Metrics {
        Metrics {
            tcp: AtomicU64::new(0),
            udp: AtomicU64::new(0),
            icmp: AtomicU64::new(0),
            other: AtomicU64::new(0),
        }
    }

    fn increment(&self, bucket: ProtocolNumber) {
        let counter = match bucket {
            ProtocolNumbers::Tcp => &self.tcp,
            ProtocolNumbers::Udp => &self.udp,
            ProtocolNumbers::Icmpv4 | ProtocolNumbers::Icmpv6 => &self.icmp,
            _ => &self.other,
        };
        counter.fetch_add(1, Ordering::Relaxed);
    }

    fn print(&self) {
        println!(
            "OVERALL: TCP {}, UDP {}, ICMP {}, OTHER {}",
            self.tcp.load(Ordering::Relaxed),
            self.udp.load(Ordering::Relaxed),
            self.icmp.load(Ordering::Relaxed),
            self.other.load(Ordering::Relaxed)
        );
    }
}

lazy_static! {
    static ref METRICS: Metrics = Metrics::new();
}

fn install<T, S>(ports: Vec<T>, sched: &mut S)
where
    T: PacketRx + PacketTx + Display + Clone + 'static,
    S: Scheduler + Sized,
{
    let pipelines: Vec<_> = ports
        .iter()
        .map(|port| {
            ReceiveBatch::new(port.clone())
                .map(count_packets)
                .send(port.clone())
        })
        .collect();

    println!("Running {} pipelines", pipelines.len());

    for pipeline in pipelines {
        sched.add_task(pipeline).unwrap();
    }
}

fn count_packets(packet: RawPacket) -> Result<RawPacket> {
    let ethernet = packet.parse::<Ethernet>()?;
    match ethernet.ether_type() {
        EtherTypes::Ipv4 => {
            let ipv4 = ethernet.parse::<Ipv4>()?;
            METRICS.increment(ipv4.protocol());
            Ok(ipv4.reset())
        }
        EtherTypes::Ipv6 => {
            let ipv6 = ethernet.parse::<Ipv6>()?;
            METRICS.increment(ipv6.next_header());
            Ok(ipv6.reset())
        }
        _ => Ok(ethernet.reset()),
    }
}

fn main() -> Result<()> {
    let configuration = load_config()?;
    println!("{}", configuration);
    let mut runtime = Runtime::init(&configuration)?;
    runtime.add_pipeline_to_run(install);
    runtime.add_task_to_run(|| METRICS.print(), Duration::from_secs(1));
    runtime.execute()
}
