extern crate e2d2;
extern crate getopts;
use getopts::Options;
use e2d2::scheduler::*;
use std::env;
use std::process;
fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optopt("", "config", "Configuration file", "TOML file");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };

    if matches.opt_present("h") {
        print!("{}", opts.usage(&format!("Usage: {} [options]", program)));
        process::exit(0)
    }

    let cfg = matches.opt_str("config").expect("No configuration supplied, rendering this meaningless");
    let sched_cfg = read_configuration(&cfg[..]).expect("Could not parse configuration");
    println!("Read configuration {}", sched_cfg)
}
