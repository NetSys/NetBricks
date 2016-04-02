#![feature(box_syntax)]
extern crate e2d2;
extern crate fnv;
extern crate time;
extern crate simd;
extern crate getopts;
extern crate rand;
use e2d2::io::*;
use e2d2::headers::*;
use e2d2::utils::*;
use e2d2::packet_batch::*;
use e2d2::state::*;
use fnv::FnvHasher;
use getopts::Options;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::env;
use std::time::Duration;
use std::thread;

const CONVERSION_FACTOR: f64 = 1000000000.;
type FnvHash = BuildHasherDefault<FnvHasher>;

fn monitor<T: 'static + Batch>(parent: T, mut monitoring_cache: MergeableStoreDP<isize>) -> CompositionBatch {
    parent.parse::<MacHeader>()
          .transform(box move |hdr, payload, _| {
              // No one else should be writing to this, so I think relaxed is safe here.
              let src = hdr.src.clone();
              hdr.src = hdr.dst;
              hdr.dst = src;
              monitoring_cache.update(ipv4_extract_flow(payload), 1);
          })
          .parse::<IpHeader>()
          .transform(box |hdr, _, _| {
              let ttl = hdr.ttl();
              hdr.set_ttl(ttl + 1)
          })
          .compose()
}

fn recv_thread(ports: Vec<PmdPort>, queue: i32, core: i32, counter: MergeableStoreDP<isize>) {
    init_thread(core, core);
    println!("Receiving started");

    let pipelines: Vec<_> = ports.iter()
                                 .map(|port| {
                                     let ctr = counter.clone();
                                     monitor(ReceiveBatch::new(port.copy(), queue), ctr)
                                         .send(port.copy(), queue)
                                         .compose()
                                 })
                                 .collect();
    println!("Running {} pipelines", pipelines.len());
    let mut combined = merge(pipelines);
    loop {
        combined.process();
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optmulti("w", "whitelist", "Whitelist PCI", "PCI");
    opts.optmulti("c", "core", "Core to use", "core");
    opts.optflag("h", "help", "print this help menu");
    opts.optopt("m", "master", "Master core", "master");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };
    if matches.opt_present("h") {
        print!("{}", opts.usage(&format!("Usage: {} [options]", program)));
    }
    let cores_str = matches.opt_strs("c");
    let master_core = matches.opt_str("m")
                             .unwrap_or_else(|| String::from("0"))
                             .parse()
                             .expect("Could not parse master core spec");
    println!("Using master core {}", master_core);

    let whitelisted = matches.opt_strs("w");
    if cores_str.len() > whitelisted.len() {
        println!("More cores than ports");
        std::process::exit(1);
    }
    let cores: Vec<i32> = cores_str.iter()
                                   .map(|n: &String| n.parse().ok().expect(&format!("Core cannot be parsed {}", n)))
                                   .collect();
    for (core, wl) in cores.iter().zip(whitelisted.iter()) {
        println!("Going to use core {} for wl {}", core, wl);
    }
    let mut core_map = HashMap::<i32, Vec<i32>>::with_capacity(cores.len());
    for (core, port) in cores.iter().zip(0..whitelisted.len()) {
        {
            match core_map.get(&core) {
                Some(_) => core_map.get_mut(&core).expect("Incorrect logic").push(port as i32),
                None => {
                    core_map.insert(core.clone(), vec![port as i32]);
                    ()
                }
            }
        }
    }

    init_system_wl(&format!("recv{}", cores_str.join("")),
                   master_core,
                   &whitelisted);
    let ports_by_core: HashMap<_, _> = core_map.iter()
                                               .map(|(core, ports)| {
                                                   let c = core.clone();
                                                   let recv_ports: Vec<_> =
                                                       ports.iter()
                                                            .map(|p| {
                                                                PmdPort::new_mq_port(p.clone() as i32, 1, 1, &[c], &[c])
                                                                    .expect("Could not initialize port")
                                                            })
                                                            .collect();
                                                   (c, recv_ports)
                                               })
                                               .collect();
    const _BATCH: usize = 1 << 10;
    const _CHANNEL_SIZE: usize = 256;
    let mut consumer = MergeableStoreCP::new();
    let _thread: Vec<_> = ports_by_core.iter()
                                       .map(|(core, ports)| {
                                           let c = core.clone();
                                           let mon = consumer.dp_store();
                                           let p: Vec<_> = ports.iter().map(|p| p.copy()).collect();
                                           std::thread::spawn(move || recv_thread(p, 0, c, mon))
                                       })
                                       .collect();
    let mut pkts_so_far = (0, 0);
    let mut start = time::precise_time_ns() as f64 / CONVERSION_FACTOR;
    let sleep_time = Duration::from_millis(500);
    loop {
        thread::sleep(sleep_time); // Sleep for a bit
        consumer.sync();
        let now = time::precise_time_ns() as f64 / CONVERSION_FACTOR;
        if now - start > 1.0 {
            let pkts = ports_by_core.values()
                                    .map(|pvec| {
                                        pvec.iter()
                                            .map(|p| p.stats(0))
                                            .fold((0, 0), |(r, t), (rp, tp)| (r + rp, t + tp))
                                    })
                                    .fold((0, 0), |(r, t), (rp, tp)| (r + rp, t + tp));
            println!("{:.2} OVERALL RX {:.2} TX {:.2} FLOWS {}",
                     now - start,
                     (pkts.0 - pkts_so_far.0) as f64 / (now - start),
                     (pkts.1 - pkts_so_far.1) as f64 / (now - start),
                     consumer.len());
            start = now;
            pkts_so_far = pkts;
        }
    }
}
