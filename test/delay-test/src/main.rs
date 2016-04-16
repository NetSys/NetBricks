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
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("", "secondary", "run as a secondary process");
    opts.optopt("n", "name", "name to use for the current process", "name");
    opts.optmulti("w", "whitelist", "Whitelist PCI", "PCI");
    opts.optmulti("v", "vdevs", "Virtual Devices to add", "PCI");
    opts.optmulti("c", "core", "Core to use", "core");
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
    let name = matches.opt_str("n").unwrap_or_else(|| String::from("recv"));



    let cores: Vec<i32> = cores_str.iter()
                                   .map(|n: &String| n.parse().ok().expect(&format!("Core cannot be parsed {}", n)))
                                   .collect();

    let ports = if matches.opt_present("secondary") {
        let vdevs = matches.opt_strs("v");
        if cores.len() > vdevs.len() {
            println!("More cores than vdevs");
            std::process::exit(1);
        }
        init_system_secondary(&name,
                              master_core,
                              &[]);
        // Fix this so we don't assume Bess.
        let mut ports = Vec::with_capacity(vdevs.len());
        for (core, vdev) in cores.iter().zip(vdevs.iter()) {
            ports.push(PmdPort::new_vdev(vdev, *core).expect("Could not initialize vdev"))
        }
        ports
    } else {
        let whitelisted = matches.opt_strs("w");
        if cores.len() > whitelisted.len() {
            println!("More cores than ports");
            std::process::exit(1);
        }
        init_system_wl(&name,
                       master_core,
                       &whitelisted);
        let mut ports = Vec::with_capacity(whitelisted.len());
        for (core, wl) in cores.iter().zip(whitelisted.iter()) {
            println!("Going to use core {} for wl {}", core, wl);
        }
        for (core, port) in cores.iter().zip(0..whitelisted.len()) {
            ports.push(PmdPort::new_mq_port(port as i32, 1, 1, &[*core], &[*core])
                       .expect("Could not initialize port"))
        }
        ports
    };

    let mut ports_by_core = HashMap::<i32, Vec<PmdPort>>::with_capacity(cores.len());
    for (core, port) in cores.iter().zip(ports.iter()) {
        {
            ports_by_core.entry(*core).or_insert(vec![]).push(port.copy());
        }
    }
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
    let mut last_printed = 0.;
    const MAX_PRINT_INTERVAL : f64 = 15.;
    let mut start = time::precise_time_ns() as f64 / CONVERSION_FACTOR;
    let sleep_time = Duration::from_millis(5000);
    println!("0 OVERALL RX 0.00 TX 0.00 CYCLE_PER_DELAY 0 0 0");
    loop {
        thread::sleep(sleep_time); // Sleep for a bit
        let now = time::precise_time_ns() as f64 / CONVERSION_FACTOR;
        if now - start > 10.0 {
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
            let rx_pkts = pkts.0 - pkts_so_far.0;
            if rx_pkts > 0 || now - last_printed > MAX_PRINT_INTERVAL {
                println!("{:.2} OVERALL RX {:.2} TX {:.2} CYCLE_PER_DELAY {} {} {}",
                         now - start,
                         rx_pkts as f64 / (now - start),
                         (pkts.1 - pkts_so_far.1) as f64 / (now - start),
                         start_cycles,
                         end_cycles,
                         delay);
                last_printed = now;
            }
            start = now;
            pkts_so_far = pkts;
        }
    }
}
