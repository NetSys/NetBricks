#![feature(box_syntax)]
#![feature(asm)]
extern crate e2d2;
extern crate fnv;
extern crate time;
extern crate getopts;
extern crate rand;

use e2d2::config::*;
use e2d2::interface::*;
use e2d2::native::zcsi::*;
use getopts::Options;
use std::env;
use std::process;

pub mod bessnb;
mod nf;

const CONVERSION_FACTOR: f64 = 1000000000.;
const GATE_PKT_QUEUE: usize = 32;

fn alloc_gates(num_gates: usize) -> Vec<bessnb::BessGate> {
    let mut gates = Vec::<bessnb::BessGate>::new();

    for _ in 0..num_gates {
        let mut pkts = Vec::<*mut mbuf::MBuf>::with_capacity(GATE_PKT_QUEUE);
        unsafe {
            pkts.set_len(GATE_PKT_QUEUE);
        }

        let buf = bessnb::BessGate {
            capacity: GATE_PKT_QUEUE,
            cnt: 0,
            pkts: pkts.as_mut_ptr(),
        };

        gates.push(buf);
    }

    gates
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();

    opts.optflag("h", "help", "print this help menu");
    opts.optopt("g", "gate", "# of input/output gates", "gates");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };

    if matches.opt_present("h") {
        print!("{}", opts.usage(&format!("Usage: {} [options]", program)));
        process::exit(0)
    }

    let gate_arg: usize = matches.opt_str("g")
        .unwrap_or_else(|| String::from("1"))
        .parse()
        .expect("Could not parse gate");

    let name = "bessmod";
    let configuration = NetbricksConfiguration::new_with_name(&name[..]);

    println!("Going to start with configuration {}", configuration);
    dpdk::init_system(&configuration);


    let mut rx_gates = alloc_gates(gate_arg);
    let mut tx_gates = alloc_gates(gate_arg);

    let mut rx_gate_ptrs = Vec::<*mut bessnb::BessGate>::new();
    let mut tx_gate_ptrs = Vec::<*mut bessnb::BessGate>::new();
    for i in 0..gate_arg {
        rx_gate_ptrs.push(&mut rx_gates[i]);
        tx_gate_ptrs.push(&mut tx_gates[i]);
    }

    let ctx = bessnb::init_mod(gate_arg,
                               rx_gate_ptrs.as_mut_ptr(),
                               tx_gate_ptrs.as_mut_ptr());

    let mut pkts_so_far = (0, 0);
    let start = time::precise_time_ns() as f64 / CONVERSION_FACTOR;

    println!("0 OVERALL RX 0.00 TX 0.00");

    // Run the processing loop in the current thread, as if we are BESS
    let mut last = start;
    loop {
        // feed the module with a randomly chosen input gate
        unsafe {
            let i = rand::random::<usize>() % gate_arg;
            assert!(rx_gates[i].cnt == 0);
            let ret = mbuf_alloc_bulk(rx_gates[i].pkts, 60, GATE_PKT_QUEUE as i32);
            assert!(ret == 0, "Packet allocation failed");
            rx_gates[i].cnt = GATE_PKT_QUEUE;
        }

        bessnb::run_once(ctx);

        // flush all output gates
        unsafe {
            for i in 0..gate_arg {
                if tx_gates[i].cnt > 0 {
                    mbuf_free_bulk(tx_gates[i].pkts, tx_gates[i].cnt as i32);
                }
                tx_gates[i].cnt = 0;
            }
        }

        let now = time::precise_time_ns() as f64 / CONVERSION_FACTOR;
        if now - last > 1. {
            let pkts = bessnb::get_stats(ctx);
            let rx_pkts = pkts.0 - pkts_so_far.0;
            let tx_pkts = pkts.1 - pkts_so_far.1;

            println!("{:.2} OVERALL RX {:.2} TX {:.2}",
                     now - start,
                     rx_pkts as f64 / (now - last),
                     tx_pkts as f64 / (now - last));
            last = now;
            pkts_so_far = pkts;
        }
    }
}
