extern crate fnv;
#[macro_use]
extern crate lazy_static;
extern crate netbricks;
use fnv::FnvHasher;
use netbricks::allocators::CacheAligned;
use netbricks::common::Result;
use netbricks::config::load_config;
use netbricks::interface::*;
use netbricks::operators::{Batch, ReceiveBatch};
use netbricks::packets::ip::v4::Ipv4;
use netbricks::packets::ip::Flow;
use netbricks::packets::{Ethernet, Packet, Udp};
use netbricks::runtime::Runtime;
use netbricks::scheduler::Scheduler;
use netbricks::utils::cidr::v4::Ipv4Cidr;
use netbricks::utils::cidr::Cidr;
use std::collections::HashSet;
use std::hash::BuildHasherDefault;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::sync::RwLock;

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

fn install<S: Scheduler + Sized>(ports: Vec<CacheAligned<PortQueue>>, sched: &mut S) {
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

fn main() -> Result<()> {
    let configuration = load_config()?;
    println!("{}", configuration);
    let mut runtime = Runtime::init(&configuration)?;
    runtime.add_pipeline_to_run(install);
    runtime.execute()
}
