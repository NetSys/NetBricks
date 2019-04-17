#![feature(box_syntax)]
#![feature(asm)]
extern crate netbricks;
use netbricks::common::Result;
use netbricks::config::{basic_opts, read_matches};
use netbricks::interface::*;
use netbricks::new_operators::{mpsc_batch, Batch, Enqueue, MpscProducer, ReceiveBatch};
use netbricks::packets::icmp::v6::{EchoReply, EchoRequest, Icmpv6};
use netbricks::packets::ip::v6::Ipv6;
use netbricks::packets::ip::ProtocolNumbers;
use netbricks::packets::{EtherTypes, Ethernet, Packet, RawPacket};
use netbricks::scheduler::*;
use std::env;
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
    println!("Echo reply pipeline");

    let (producer, outbound) = mpsc_batch();
    let outbound = outbound.send(ports[0].clone());
    sched.add_task(outbound).unwrap();

    println!("Receiving started");

    let pipelines: Vec<_> = ports
        .iter()
        .map(move |port| {
            let producer = producer.clone();
            ReceiveBatch::new(port.clone())
                .map(move |p| reply_echo(p, &producer))
                .filter(|_| false)
                .send(port.clone())
        })
        .collect();

    println!("Running {} pipelines", pipelines.len() + 1);

    for pipeline in pipelines {
        sched.add_task(pipeline).unwrap();
    }
}

fn reply_echo(packet: RawPacket, producer: &MpscProducer) -> Result<Icmpv6<Ipv6, EchoRequest>> {
    let reply = RawPacket::new()?;

    let ethernet = packet.parse::<Ethernet>()?;
    let mut reply = reply.push::<Ethernet>()?;
    reply.set_src(ethernet.dst());
    reply.set_dst(ethernet.src());
    reply.set_ether_type(EtherTypes::Ipv6);

    let ipv6 = ethernet.parse::<Ipv6>()?;
    let mut reply = reply.push::<Ipv6>()?;
    reply.set_src(ipv6.dst());
    reply.set_dst(ipv6.src());
    reply.set_next_header(ProtocolNumbers::Icmpv6);

    let icmpv6 = ipv6.parse::<Icmpv6<Ipv6, ()>>()?;
    let echo = icmpv6.downcast::<EchoRequest>()?;
    let mut reply = reply.push::<Icmpv6<Ipv6, EchoReply>>()?;
    reply.set_identifier(echo.identifier());
    reply.set_seq_no(echo.seq_no());
    reply.set_data(echo.data())?;
    reply.cascade();

    producer.enqueue(reply.reset());

    Ok(echo)
}

fn main() {
    let mut opts = basic_opts();
    opts.optopt(
        "",
        "dur",
        "Test duration",
        "If this option is set to a nonzero value, then the \
         test will just loop after 2 seconds",
    );

    let args: Vec<String> = env::args().collect();
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };

    let configuration = read_matches(&matches, &opts);

    let test_duration: u64 = matches
        .opt_str("dur")
        .unwrap_or_else(|| String::from("0"))
        .parse()
        .expect("Could not parse test duration");

    match initialize_system(&configuration) {
        Ok(mut context) => {
            context.start_schedulers();
            context.add_pipeline_to_run(Arc::new(move |p, s: &mut StandaloneScheduler| test(p, s)));
            context.execute();

            if test_duration != 0 {
                thread::sleep(Duration::from_secs(test_duration));
            } else {
                loop {
                    thread::sleep(Duration::from_secs(2));
                }
            }
        }
        Err(ref e) => {
            println!("Error: {:?}", e);
            process::exit(1);
        }
    }
}
