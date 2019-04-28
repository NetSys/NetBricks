extern crate netbricks;
use netbricks::common::Result;
use netbricks::config::get_config;
use netbricks::interface::*;
use netbricks::operators::{Batch, ReceiveBatch};
use netbricks::packets::{Ethernet, Packet, RawPacket};
use netbricks::scheduler::*;
use std::fmt::Display;
use std::process;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn test<T, S>(ports: Vec<T>, sched: &mut S)
where
    T: PacketRx + PacketTx + Display + Clone + 'static,
    S: Scheduler + Sized,
{
    for port in &ports {
        println!("Receiving port {}", port);
    }

    let pipelines: Vec<_> = ports
        .iter()
        .map(|port| {
            ReceiveBatch::new(port.clone())
                .map(macswap)
                .send(port.clone())
        })
        .collect();

    println!("Running {} pipelines", pipelines.len());
    for pipeline in pipelines {
        sched.add_task(pipeline).unwrap();
    }
}

fn macswap(packet: RawPacket) -> Result<Ethernet> {
    assert!(packet.refcnt() == 1);
    let mut ethernet = packet.parse::<Ethernet>()?;
    ethernet.swap_addresses();
    Ok(ethernet)
}

fn main() {
    let configuration = get_config().unwrap();
    println!("{}", configuration);

    let test_duration: u64 = 5;

    match initialize_system(&configuration) {
        Ok(mut context) => {
            context.start_schedulers();

            context.add_pipeline_to_run(Arc::new(move |p, s: &mut StandaloneScheduler| test(p, s)));
            context.execute();

            if test_duration != 0 {
                thread::sleep(Duration::from_secs(test_duration));
            } else {
                loop {
                    thread::sleep(Duration::from_secs(1));
                }
            }
        }
        Err(ref e) => {
            println!("Error: {:?}", e);
            process::exit(1);
        }
    }
}
