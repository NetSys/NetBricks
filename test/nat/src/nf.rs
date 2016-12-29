use e2d2::headers::*;
use e2d2::operators::*;
use e2d2::utils::*;
use e2d2::scheduler::*;
use fnv::FnvHasher;
use std::net::Ipv4Addr;
use std::convert::From;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;

#[derive(Clone, Default)]
struct Unit;
#[derive(Clone, Copy, Default)]
struct FlowUsed {
    pub flow: Flow,
    pub time: u64,
    pub used: bool,
}

type FnvHash = BuildHasherDefault<FnvHasher>;
pub fn nat<T: 'static + Batch<Header = NullHeader>>(parent: T,
                                                    _s: &mut Scheduler,
                                                    nat_ip: &Ipv4Addr)
                                                    -> CompositionBatch {
    let ip = u32::from(*nat_ip);
    let mut port_hash = HashMap::<Flow, Flow, FnvHash>::with_capacity_and_hasher(65536, Default::default());
    let mut flow_vec: Vec<FlowUsed> = (MIN_PORT..65535).map(|_| Default::default()).collect();
    let mut next_port = 1024;
    const MIN_PORT: u16 = 1024;
    const MAX_PORT: u16 = 65535;
    let pipeline = parent.parse::<MacHeader>()
        .transform(box move |pkt| {
            // let hdr = pkt.get_mut_header();
            let payload = pkt.get_mut_payload();
            if let Some(flow) = ipv4_extract_flow(payload) {
                let found = match port_hash.get(&flow) {
                    Some(s) => {
                        s.ipv4_stamp_flow(payload);
                        true
                    }
                    None => false,
                };
                if !found {
                    if next_port < MAX_PORT {
                        let assigned_port = next_port; //FIXME.
                        next_port += 1;
                        flow_vec[assigned_port as usize].flow = flow;
                        flow_vec[assigned_port as usize].used = true;
                        let mut outgoing_flow = flow.clone();
                        outgoing_flow.src_ip = ip;
                        outgoing_flow.src_port = assigned_port;
                        let rev_flow = outgoing_flow.reverse_flow();

                        port_hash.insert(flow, outgoing_flow);
                        port_hash.insert(rev_flow, flow.reverse_flow());

                        outgoing_flow.ipv4_stamp_flow(payload);
                    }
                }
            }
        });
    pipeline.compose()
}
