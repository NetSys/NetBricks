#![feature(box_syntax)]
extern crate e2d2;
extern crate time;
extern crate simd;
extern crate getopts;
extern crate rand;
use e2d2::io;
use e2d2::io::*;
use e2d2::headers::*;
use e2d2::utils::*;
use getopts::Options;
use std::collections::HashMap;
use std::env;
use std::time::Duration;
use std::thread;
use std::any::Any;
//use std::net::Ipv4Addr;

const CONVERSION_FACTOR: u64 = 1000000000;

fn monitor<T: 'static + Batch>(parent: T) -> CompositionBatch {
    let f = box |hdr: &mut MacHeader, _: &mut [u8], _: Option<&mut Any>| {
        let src = hdr.src.clone();
        hdr.src = hdr.dst;
        hdr.dst = src;
    };

    parent//.context::<Flow>()
          .parse::<MacHeader>()
          .transform(f)
          .map(box |_, payload, _| {
              //let flow = ctx.unwrap().downcast_mut::<Flow>().expect("Wrong type");
              //*flow = 
              ipv4_extract_flow(payload);
          })
          .parse::<IpHeader>()
          .transform(box |hdr, _, _| {
              let ttl = hdr.ttl();
              hdr.set_ttl(ttl + 1)
          })
          .compose()
    //parent.context::<Flow>()
          //.parse::<MacHeader>()
          //.transform(f)
          //.parse::<IpHeader>()
          //.map(box |hdr, ctx| {
              //match ctx {
                  //Some(x) => {
                      //let s = x.downcast_mut::<Flow>().expect("Wrong type");
                      //s.src_ip = hdr.src();
                      //s.dst_ip = hdr.dst();
                      //s.proto = hdr.protocol();
                  //}
                  //None => panic!("no context"),
              //}
          //})
          //.parse::<UdpHeader>()
          //.filter(box |hdr, _| hdr.src_port() != 21 && hdr.dst_port() != 21)
          //.map(box |hdr, ctx| {
              //match ctx {
                  //Some(x) => {
                      //let s = x.downcast_mut::<Flow>().expect("Wrong type");
                      //s.src_port = hdr.src_port();
                      //s.dst_port = hdr.dst_port();
                  //}
                  //None => panic!("no context"),
              //}
          //})
          //.compose()
}

fn recv_thread(ports: Vec<io::PmdPort>, queue: i32, core: i32) {
    io::init_thread(core, core);
    println!("Receiving started");

    let pipelines: Vec<_> = ports.iter()
                                 .map(|port| {
                                     monitor(io::ReceiveBatch::new(port.copy(), queue))
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

    io::init_system_wl(&format!("recv{}", cores_str.join("")),
                       master_core,
                       &whitelisted);
    let ports_by_core: HashMap<_, _> = core_map.iter()
                                               .map(|(core, ports)| {
                                                   let c = core.clone();
                                                   let recv_ports: Vec<_> =
                                                       ports.iter()
                                                            .map(|p| {
                                                                io::PmdPort::new_mq_port(p.clone() as i32,
                                                                                         1,
                                                                                         1,
                                                                                         &[c],
                                                                                         &[c])
                                                                    .expect("Could not initialize port")
                                                            })
                                                            .collect();
                                                   (c, recv_ports)
                                               })
                                               .collect();
    let _thread: Vec<_> = ports_by_core.iter()
                                       .map(|(core, ports)| {
                                           let c = core.clone();
                                           let p: Vec<_> = ports.iter().map(|p| p.copy()).collect();
                                           std::thread::spawn(move || recv_thread(p, 0, c))
                                       })
                                       .collect();
    let mut pkts_so_far = (0, 0);
    let mut start = time::precise_time_ns() / CONVERSION_FACTOR;
    let sleep_time = Duration::from_secs(1);
    loop {
        thread::sleep(sleep_time); // Sleep for a bit
        let now = time::precise_time_ns() / CONVERSION_FACTOR;
        let pkts = ports_by_core.values()
                                .map(|pvec| {
                                    pvec.iter()
                                        .map(|p| p.stats(0))
                                        .fold((0, 0), |(r, t), (rp, tp)| (r + rp, t + tp))
                                })
                                .fold((0, 0), |(r, t), (rp, tp)| (r + rp, t + tp));
        println!("{} OVERALL RX {} TX {}",
                 now - start,
                 pkts.0 - pkts_so_far.0,
                 pkts.1 - pkts_so_far.1);
        start = now;
        pkts_so_far = pkts;
    }
}
