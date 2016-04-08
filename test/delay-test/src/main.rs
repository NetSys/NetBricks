#![feature(box_syntax)]
#![feature(asm)]
extern crate e2d2;
extern crate fnv;
extern crate time;
extern crate simd;
extern crate getopts;
extern crate rand;
use e2d2::io::*;
use e2d2::headers::*;
use e2d2::packet_batch::*;
use getopts::Options;
use std::collections::HashMap;
use std::env;
use std::time::Duration;
use std::thread;

const CONVERSION_FACTOR: f64 = 1000000000.;

#[inline]
fn rdtscp_unsafe() -> u64 {
    unsafe {
        let low: u32;
        let high: u32;
        asm!("rdtscp"
             : "={eax}" (low), "={edx}" (high)
             :
             : "{ecx}"
             : "volatile");
        ((high as u64) << 32) | (low as u64)
    }
}

#[inline]
fn rdtscp() -> u64 {
    rdtscp_unsafe()
}

#[inline]
fn lat() {
    unsafe {
        asm!("nop"
             :
             :
             :
             : "volatile");
    }
}

#[inline]
fn delay_loop(delay: u64) {
    let mut d = 0;
    while d < delay {
        lat();
        d += 1;
    }
}

fn delay<T: 'static + Batch>(parent: T, delay: u64) -> CompositionBatch {
    parent.parse::<MacHeader>()
          .transform(box move |hdr, _, _| {
              let src = hdr.src.clone();
              hdr.src = hdr.dst;
              hdr.dst = src;
              delay_loop(delay);
          })
          .compose()
}

fn recv_thread(ports: Vec<PmdPort>, queue: i32, core: i32, delay_arg: u64) {
    init_thread(core, core);
    println!("Receiving started");

    let pipelines: Vec<_> = ports.iter()
                                 .map(|port| {
                                     delay(ReceiveBatch::new(port.copy(), queue), delay_arg)
                                         .send(port.copy(), queue)
                                         .compose()
                                 }).collect();
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
    opts.optopt("d", "delay", "Delay cycles", "cycles");
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
    let delay_arg = matches.opt_str("d")
                       .unwrap_or_else(|| String::from("100"))
                       .parse()
                       .expect("Could not parse delay");
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
    let _thread: Vec<_> = ports_by_core.iter()
                                       .map(|(core, ports)| {
                                           let c = core.clone();
                                           let p: Vec<_> = ports.iter().map(|p| p.copy()).collect();
                                           std::thread::spawn(move || recv_thread(p, 0, c, delay_arg))
                                       })
                                       .collect();
    let mut pkts_so_far = (0, 0);
    let mut start = time::precise_time_ns() as f64 / CONVERSION_FACTOR;
    let sleep_time = Duration::from_millis(500);
    loop {
        thread::sleep(sleep_time); // Sleep for a bit
        let now = time::precise_time_ns() as f64 / CONVERSION_FACTOR;
        if now - start > 1.0 {
            let pkts = ports_by_core.values()
                                    .map(|pvec| {
                                        pvec.iter()
                                            .map(|p| p.stats(0))
                                            .fold((0, 0), |(r, t), (rp, tp)| (r + rp, t + tp))
                                    })
                                    .fold((0, 0), |(r, t), (rp, tp)| (r + rp, t + tp));
            let start_cycles = rdtscp();
            delay_loop(100);
            let end_cycles = rdtscp();
            let delay = end_cycles - start_cycles;
            println!("{:.2} OVERALL RX {:.2} TX {:.2} CYCLE_PER_DELAY {} {} {}",
                     now - start,
                     (pkts.0 - pkts_so_far.0) as f64 / (now - start),
                     (pkts.1 - pkts_so_far.1) as f64 / (now - start),
                     start_cycles,
                     end_cycles,
                     delay);
            start = now;
            pkts_so_far = pkts;
        }
    }
}
