extern crate netbricks;
use netbricks::common::Result;
use netbricks::config::load_config;
use netbricks::interface::{PacketRx, PacketTx};
use netbricks::operators::{mpsc_batch, Batch, Enqueue, MpscProducer, ReceiveBatch};
use netbricks::packets::icmp::v6::{EchoReply, EchoRequest, Icmpv6};
use netbricks::packets::ip::v6::Ipv6;
use netbricks::packets::ip::ProtocolNumbers;
use netbricks::packets::{EtherTypes, Ethernet, Packet, RawPacket};
use netbricks::runtime::Runtime;
use netbricks::scheduler::Scheduler;
use std::fmt::Display;

fn install<T, S>(ports: Vec<T>, sched: &mut S)
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

fn main() -> Result<()> {
    let configuration = load_config()?;
    println!("{}", configuration);
    let mut runtime = Runtime::init(&configuration)?;
    runtime.add_pipeline_to_run(install);
    runtime.execute()
}
