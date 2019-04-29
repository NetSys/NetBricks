#![feature(box_syntax)]
#![feature(asm)]
extern crate fnv;
extern crate getopts;
extern crate netbricks;
extern crate rand;
extern crate time;
#[macro_use]
extern crate lazy_static;
use fnv::FnvHasher;
use netbricks::allocators::CacheAligned;
use netbricks::config::{basic_opts, read_matches};
use netbricks::interface::*;
use netbricks::operators::{Batch, ReceiveBatch};
use netbricks::packets::ip::v4::Ipv4;
use netbricks::packets::ip::Flow;
use netbricks::packets::{Ethernet, Packet, Udp};
use netbricks::scheduler::*;
use netbricks::utils::cidr::v4::Ipv4Cidr;
use netbricks::utils::cidr::Cidr;
use std::collections::HashSet;
use std::env;
use std::hash::BuildHasherDefault;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;
use std::time::Duration;

const CONVERSION_FACTOR: f64 = 1000000000.;

type FnvHash = BuildHasherDefault<FnvHasher>;

lazy_static! {
    static ref FLOW_CACHE: Arc<RwLock<HashSet<Flow, FnvHash>>> = {
        let m = HashSet::with_hasher(Default::default());
        Arc::new(RwLock::new(m))
    };
}

lazy_static! {
    static ref ACLS: Arc<RwLock<Vec<Acl>>> = {
        let acl = vec![Acl {
            src_prefix: Some(Ipv4Cidr::new(Ipv4Addr::new(0, 0, 0, 0), 0).unwrap()),
            dst_prefix: None,
            src_port: None,
            dst_port: None,
            established: None,
            drop: false,
        }];
        Arc::new(RwLock::new(acl))
    };
}

#[derive(Clone)]
pub struct Acl {
    pub src_prefix: Option<Ipv4Cidr>,
    pub dst_prefix: Option<Ipv4Cidr>,
    pub src_port: Option<u16>,
    pub dst_port: Option<u16>,
    pub established: Option<bool>,
    // TODO: Related not complete
    pub drop: bool,
}

impl Acl {
    fn contains(&self, ip: IpAddr) -> bool {
        if let Some(ref prefix) = self.src_prefix {
            prefix.contains_ip(ip)
        } else {
            true
        }
    }

    fn matches(&self, flow: &Flow) -> bool {
        if self.contains(flow.src_ip())
            && self.contains(flow.dst_ip())
            && (self.src_port.is_none() || flow.src_port() == self.src_port.unwrap())
            && (self.dst_port.is_none() || flow.dst_port() == self.dst_port.unwrap())
        {
            if let Some(established) = self.established {
                let rev_flow = flow.reverse();
                (FLOW_CACHE.read().unwrap().contains(flow)
                    || FLOW_CACHE.read().unwrap().contains(&rev_flow))
                    == established
            } else {
                true
            }
        } else {
            false
        }
    }
}

fn test<S: Scheduler + Sized>(ports: Vec<CacheAligned<PortQueue>>, sched: &mut S) {
    for port in &ports {
        println!(
            "Receiving port {} rxq {} txq {}",
            port.port.mac_address(),
            port.rxq(),
            port.txq()
        );
    }

    let pipelines: Vec<_> = ports
        .iter()
        .map(|port| {
            ReceiveBatch::new(port.clone())
                .map(|p| {
                    let mut ethernet = p.parse::<Ethernet>()?;
                    ethernet.swap_addresses();
                    let v4 = ethernet.parse::<Ipv4>()?;
                    let udp = v4.parse::<Udp<Ipv4>>()?;
                    Ok(udp)
                })
                .filter(|p| acl_match(p))
                .send(port.clone())
        })
        .collect();

    println!("Running {} pipelines", pipelines.len());
    for pipeline in pipelines {
        sched.add_task(pipeline).unwrap();
    }
}

fn acl_match(p: &Udp<Ipv4>) -> bool {
    let flow = p.flow();
    let acls = ACLS.read().unwrap();
    let matches = acls.iter().find(|ref acl| acl.matches(&flow));

    if let Some(acl) = matches {
        if !acl.drop {
            FLOW_CACHE.write().unwrap().insert(flow);
        }
        true
    } else {
        false
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let opts = basic_opts();
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };
    let configuration = read_matches(&matches, &opts);

    let mut config = initialize_system(&configuration).unwrap();
    config.start_schedulers();

    config.add_pipeline_to_run(Arc::new(move |p, s: &mut StandaloneScheduler| test(p, s)));
    config.execute();

    let mut pkts_so_far = (0, 0);
    let mut last_printed = 0.;
    const MAX_PRINT_INTERVAL: f64 = 30.;
    const PRINT_DELAY: f64 = 15.;
    let sleep_delay = (PRINT_DELAY / 2.) as u64;
    let mut start = time::precise_time_ns() as f64 / CONVERSION_FACTOR;
    let sleep_time = Duration::from_millis(sleep_delay);
    println!("0 OVERALL RX 0.00 TX 0.00 CYCLE_PER_DELAY 0 0 0");
    loop {
        thread::sleep(sleep_time); // Sleep for a bit
        let now = time::precise_time_ns() as f64 / CONVERSION_FACTOR;
        if now - start > PRINT_DELAY {
            let mut rx = 0;
            let mut tx = 0;
            for port in config.ports.values() {
                for q in 0..port.rxqs() {
                    let (rp, tp) = port.stats(q);
                    rx += rp;
                    tx += tp;
                }
            }
            let pkts = (rx, tx);
            let rx_pkts = pkts.0 - pkts_so_far.0;
            if rx_pkts > 0 || now - last_printed > MAX_PRINT_INTERVAL {
                println!(
                    "{:.2} OVERALL RX {:.2} TX {:.2}",
                    now - start,
                    rx_pkts as f64 / (now - start),
                    (pkts.1 - pkts_so_far.1) as f64 / (now - start)
                );
                last_printed = now;
                start = now;
                pkts_so_far = pkts;
            }
        }
    }
}
