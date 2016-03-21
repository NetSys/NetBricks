extern crate e2d2;
extern crate time;
extern crate simd;
extern crate getopts;
use e2d2::io;
use e2d2::io::Batch;
use e2d2::headers::*;
use getopts::Options;
use std::env;
use std::cell::Cell;

const CONVERSION_FACTOR: u64 = 1000000000;

fn recv_thread(port: io::PmdPort, queue: i32, core: i32) {
    io::init_thread(core, core);
    println!("Receiving started");
    let mut send_port = port.copy();
    let mut batch = io::PacketBatch::new(32);

    let recv_cell = Cell::new(0);

    let mut f = &mut |hdr: &mut MacHeader| {
        let src = hdr.src.clone();
        hdr.src = hdr.dst;
        hdr.dst = src;
        recv_cell.set(recv_cell.get() + 1);
    };
   
    let mut packets = batch.receive_batch(port, queue);
    let mut parse = packets.parse::<MacHeader>();
    let mut transform = parse.transform(f);
    let mut pipeline = transform.send(&mut send_port, queue);

    let mut cycles = 0;
    let mut rx = 0;
    let mut no_rx = 0;
    let mut start = time::precise_time_ns() / CONVERSION_FACTOR;
    loop {
        recv_cell.set(0);
        pipeline.process();
        let recv = recv_cell.get();
        rx += recv;
        cycles += 1;
        if recv == 0 {
            no_rx += 1
        }
        let now = time::precise_time_ns() / CONVERSION_FACTOR;
        if now > start {
            println!("{} rx_core {} pps {} no_rx {} loops {}",
                     (now - start),
                     core,
                     rx,
                     no_rx,
                     cycles);
            rx = 0;
            no_rx = 0;
            cycles = 0;
            start = now;
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optmulti("w", "whitelist", "Whitelist PCI", "PCI");
    opts.optmulti("c", "core", "Core to use", "core");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };
    if matches.opt_present("h") {
        print!("{}", opts.usage(&format!("Usage: {} [options]", program)));
    }
    let cores_str = matches.opt_strs("c");
    // let core:i32 = matches.opt_str("c").unwrap().parse().ok().expect("Core cannot be parsed");
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
    io::init_system_wl(&format!("recv{}", cores_str.join("")),
                       cores[0],
                       &whitelisted);
    let mut thread: Vec<std::thread::JoinHandle<()>> = cores.iter()
                                                            .zip(0..whitelisted.len())
                                                            .map(|(core, port)| {
                                                                let c = *core;
                                                                let recv_port =
                                                                    io::PmdPort::new_mq_port(port as i32,
                                                                                             1,
                                                                                             1,
                                                                                             &vec![c],
                                                                                             &vec![c])
                                                                        .unwrap();
                                                                println!("Started port {} core {}", port, c);
                                                                std::thread::spawn(move || recv_thread(recv_port, 0, c))
                                                            })
                                                            .collect();
    let _ = thread.pop().expect("No cores started").join();
}
