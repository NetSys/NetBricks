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

mod nf;
mod bessnb;

const CONVERSION_FACTOR: f64 = 1000000000.;
const BATCH_SIZE: usize = 32;

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();

    opts.optflag("h", "help", "print this help menu");
    opts.optopt("d", "delay", "Delay cycles", "cycles");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };

    if matches.opt_present("h") {
        print!("{}", opts.usage(&format!("Usage: {} [options]", program)));
        process::exit(0)
    }

    let delay_arg: u32 = matches.opt_str("d")
        .unwrap_or_else(|| String::from("100"))
        .parse()
        .expect("Could not parse delay");

    let name = "bessmod";
    let configuration = NetbricksConfiguration::new_with_name(&name[..]);

    println!("Going to start with configuration {}", configuration);
    dpdk::init_system(&configuration);


    let mut pkts_so_far = (0, 0);
    let start = time::precise_time_ns() as f64 / CONVERSION_FACTOR;

    println!("0 OVERALL RX 0.00 TX 0.00 CYCLE_PER_DELAY 0 0 0");

    let mut rx_pkts = Vec::<*mut mbuf::MBuf>::with_capacity(BATCH_SIZE);
    unsafe {
        rx_pkts.set_len(BATCH_SIZE);
    }

    let mut rx_buf = bessnb::PacketBuf {
        capacity: BATCH_SIZE,
        cnt: 0,
        pkts: rx_pkts.as_mut_ptr(),
    };

    let mut tx_pkts = Vec::<*mut mbuf::MBuf>::with_capacity(BATCH_SIZE);
    unsafe {
        tx_pkts.set_len(BATCH_SIZE);
    }

    let mut tx_buf = bessnb::PacketBuf {
        capacity: BATCH_SIZE,
        cnt: 0,
        pkts: tx_pkts.as_mut_ptr(),
    };

    let ctx = unsafe { bessnb::init_mod(&mut rx_buf, &mut tx_buf).as_mut().unwrap() };

    let mut last = start;
    loop {
        unsafe {
            mbuf_alloc_bulk(rx_buf.pkts, 60, BATCH_SIZE as i32);
            rx_buf.cnt = BATCH_SIZE;
        }
        bessnb::run_once(ctx);
        unsafe {
            if tx_buf.cnt > 0 {
                mbuf_free_bulk(tx_pkts.as_mut_ptr(), tx_buf.cnt as i32);
            }
            tx_buf.cnt = 0;
        }

        let now = time::precise_time_ns() as f64 / CONVERSION_FACTOR;
        if now - last > 1. {
            let (rx, tx) = bessnb::get_stats(ctx);
            let pkts = (rx, tx);
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
