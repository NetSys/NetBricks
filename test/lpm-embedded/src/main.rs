#![feature(box_syntax)]
extern crate e2d2;
extern crate fnv;
extern crate getopts;
extern crate rand;
extern crate time;
use self::nf::*;
use e2d2::config::*;
use e2d2::interface::*;
use e2d2::operators::*;
use e2d2::scheduler::*;
use std::process;
mod nf;

fn main() {
    let name = String::from("recv");
    let configuration = NetbricksConfiguration::new_with_name(&name[..]);
    let configuration = NetbricksConfiguration {
        primary_core: 0,
        ..configuration
    };
    match initialize_system(&configuration) {
        Ok(_) => {
            let port = VirtualPort::new(1).unwrap();
            let mut sched = embedded_scheduler::EmbeddedScheduler::new();
            let pipeline0 = lpm(
                ReceiveBatch::new(port.new_virtual_queue(1).unwrap()),
                &mut sched,
            );
            let pipeline1 = lpm(
                ReceiveBatch::new(port.new_virtual_queue(1).unwrap()),
                &mut sched,
            );
            let task = sched.add_task(merge(vec![pipeline0, pipeline1])).unwrap();
            println!("Dependencies for task {}", task);
            sched.display_dependencies(task);
        }
        Err(ref e) => {
            println!("Error: {}", e);
            if let Some(backtrace) = e.backtrace() {
                println!("Backtrace: {:?}", backtrace);
            }
            process::exit(1);
        }
    }
}
