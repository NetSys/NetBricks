#![feature(box_syntax)]
extern crate fnv;
extern crate getopts;
extern crate netbricks;
extern crate rand;
extern crate time;
#[macro_use]
extern crate lazy_static;
use fnv::FnvHasher;
use netbricks::common::Result;
use netbricks::config::{basic_opts, read_matches};
use netbricks::interface::*;
use netbricks::operators::{Batch, ReceiveBatch};
use netbricks::packets::ip::v4::Ipv4;
use netbricks::packets::ip::Flow;
use netbricks::packets::ip::ProtocolNumbers;
use netbricks::packets::{Ethernet, Packet, RawPacket, Tcp};
use netbricks::scheduler::*;
use netbricks::utils::Atom;
use std::collections::HashMap;
use std::env;
use std::fmt::Display;
use std::hash::BuildHasherDefault;
use std::net::{IpAddr, Ipv4Addr};
use std::process;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;
use std::time::Duration;

const CONVERSION_FACTOR: f64 = 1000000000.;

const MIN_PORT: u16 = 1024;
const MAX_PORT: u16 = 65535;

type FnvHash = BuildHasherDefault<FnvHasher>;

lazy_static! {
    static ref PORT_MAP: Arc<RwLock<HashMap<Flow, Flow, FnvHash>>> = {
        let m = HashMap::with_capacity_and_hasher(65536, Default::default());
        Arc::new(RwLock::new(m))
    };
}

lazy_static! {
    static ref FLOW_VEC: Arc<RwLock<Vec<FlowUsed>>> = {
        let m = (MIN_PORT..65535).map(|_| Default::default()).collect();
        Arc::new(RwLock::new(m))
    };
}

lazy_static! {
    static ref NEXT_PORT: Atom<u16> = { Atom::new(1024) };
}

#[derive(Clone, Default)]
struct Unit;

#[derive(Clone, Copy)]
struct FlowUsed {
    pub flow: Flow,
    pub time: u64,
    pub used: bool,
}

impl Default for FlowUsed {
    fn default() -> FlowUsed {
        FlowUsed {
            flow: Flow::new(
                IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                0,
                0,
                ProtocolNumbers::Tcp,
            ),
            time: 0,
            used: false,
        }
    }
}

fn test<T, S>(ports: Vec<T>, sched: &mut S)
where
    T: PacketRx + PacketTx + Display + Clone + 'static,
    S: Scheduler + Sized,
{
    println!("Receiving started");

    let pipelines: Vec<_> = ports
        .iter()
        .map(move |port| {
            ReceiveBatch::new(port.clone())
                .map(|p| nat(p, Ipv4Addr::new(10, 0, 0, 1)))
                .send(port.clone())
        })
        .collect();

    println!("Running {} pipelines", pipelines.len());
    for pipeline in pipelines {
        sched.add_task(pipeline).unwrap();
    }
}

fn nat(packet: RawPacket, nat_ip: Ipv4Addr) -> Result<Tcp<Ipv4>> {
    let ethernet = packet.parse::<Ethernet>()?;
    let v4 = ethernet.parse::<Ipv4>()?;
    let mut tcp = v4.parse::<Tcp<Ipv4>>()?;
    let flow = tcp.flow();

    match PORT_MAP.read().unwrap().get(&flow) {
        Some(s) => {
            let _ = tcp.stamp_flow(*s);
            tcp.cascade();
        }
        None => {
            if *NEXT_PORT.get() < MAX_PORT {
                let assigned_port = *NEXT_PORT.get();
                NEXT_PORT.set(*NEXT_PORT.get() + 1);

                let mut flow_vec = FLOW_VEC.write().unwrap();

                flow_vec[assigned_port as usize].flow = flow;
                flow_vec[assigned_port as usize].used = true;

                let mut outgoing_flow = flow.clone();
                outgoing_flow.set_src_ip(IpAddr::V4(nat_ip));
                outgoing_flow.set_src_port(assigned_port);
                let rev_flow = outgoing_flow.reverse_flow();

                let mut port_map = PORT_MAP.write().unwrap();
                port_map.insert(flow, outgoing_flow);
                port_map.insert(rev_flow, flow.reverse_flow());

                let _ = tcp.stamp_flow(outgoing_flow);
                tcp.cascade()
            }
        }
    }

    Ok(tcp)
}

fn main() {
    let opts = basic_opts();

    let args: Vec<String> = env::args().collect();
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };
    let configuration = read_matches(&matches, &opts);

    match initialize_system(&configuration) {
        Ok(mut context) => {
            context.start_schedulers();
            context.add_pipeline_to_run(Arc::new(move |p, s: &mut StandaloneScheduler| test(p, s)));
            context.execute();

            let mut pkts_so_far = (0, 0);
            let mut start = time::precise_time_ns() as f64 / CONVERSION_FACTOR;
            let sleep_time = Duration::from_millis(500);
            loop {
                thread::sleep(sleep_time); // Sleep for a bit
                let now = time::precise_time_ns() as f64 / CONVERSION_FACTOR;
                if now - start > 1.0 {
                    let mut rx = 0;
                    let mut tx = 0;
                    for port in context.ports.values() {
                        for q in 0..port.rxqs() {
                            let (rp, tp) = port.stats(q);
                            rx += rp;
                            tx += tp;
                        }
                    }
                    let pkts = (rx, tx);
                    println!(
                        "{:.2} OVERALL RX {:.2} TX {:.2}",
                        now - start,
                        (pkts.0 - pkts_so_far.0) as f64 / (now - start),
                        (pkts.1 - pkts_so_far.1) as f64 / (now - start)
                    );
                    start = now;
                    pkts_so_far = pkts;
                }
            }
        }
        Err(ref e) => {
            println!("Error: {:?}", e);
            process::exit(1);
        }
    }
}
