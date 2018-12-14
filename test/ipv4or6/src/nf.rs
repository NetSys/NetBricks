use colored::*;
use netbricks::headers::*;
use netbricks::operators::*;
use netbricks::scheduler::*;

#[inline]
fn tcp_ipv6_nf<T: 'static + Batch<Header = MacHeader>>(parent: T) -> CompositionBatch {
    parent
        .parse::<Ipv6Header>()
        .map(box |pkt| {
            let hdr = pkt.get_header();
            let flow = hdr.flow().unwrap();
            let payload = pkt.get_payload();
            let info_fmt = format!(
                "\nhdr {} next_header {:?} offset {}",
                hdr,
                hdr.next_header().unwrap(),
                hdr.offset()
            )
            .cyan();
            println!("{}", info_fmt);
            let payload_fmt = format!(
                "payload: {:x} {:x} {:x} {:x}",
                payload[0], payload[1], payload[2], payload[3]
            )
            .cyan();
            println!("{}", payload_fmt);
            let (src, dst) = (flow.src_port, flow.dst_port);
            let src_dst_fmt = format!("Src {} dst {}", src, dst).cyan();
            println!("{}", src_dst_fmt);
        })
        .parse::<TcpHeader<Ipv6Header>>()
        .map(box |pkt| {
            let tcp_fmt = format!("TCP header {}", pkt.get_header()).cyan();
            println!("{}", tcp_fmt);
        })
        .compose()
}

#[inline]
fn tcp_ipv4_nf<T: 'static + Batch<Header = MacHeader>>(parent: T) -> CompositionBatch {
    parent
        .parse::<Ipv4Header>()
        .map(box |pkt| {
            let hdr = pkt.get_header();
            let flow = hdr.flow().unwrap();
            let payload = pkt.get_payload();
            let info_fmt = format!("\nhdr {} offset {}", hdr, hdr.offset()).yellow();
            println!("{}", info_fmt);
            let payload_fmt = format!(
                "payload: {:x} {:x} {:x} {:x}",
                payload[0], payload[1], payload[2], payload[3]
            )
            .yellow();
            println!("{}", payload_fmt);
            let (src, dst) = (flow.src_port, flow.dst_port);
            let src_dst_fmt = format!("Src {} dst {}", src, dst).yellow();
            println!("{}", src_dst_fmt);
        })
        .parse::<TcpHeader<Ipv4Header>>()
        .map(box |pkt| {
            let tcp_fmt = format!("TCP header {}", pkt.get_header()).yellow();
            println!("{}", tcp_fmt);
        })
        .compose()
}

pub fn tcp_nf<T: 'static + Batch<Header = NullHeader>, S: Scheduler + Sized>(
    parent: T,
    sched: &mut S,
) -> CompositionBatch {
    let mut groups = parent
        .parse::<MacHeader>()
        .map(box |pkt| {
            let info_fmt = format!("hdr {}", pkt.get_header()).magenta();
            println!("{}", info_fmt);
            let payload = pkt.get_payload();
            println!("{}", format!("Payload: ").magenta());
            for p in payload {
                print!("{}", format!("{:x} ", p).magenta())
            }
        })
        .filter(box |pkt| match pkt.get_header().etype() {
            Some(EtherType::IPv4) => true,
            Some(EtherType::IPv6) => true,
            _ => false,
        })
        .group_by(
            2,
            box |pkt| match pkt.get_header().etype().unwrap() {
                EtherType::IPv4 => 1,
                _ => 0,
            },
            sched,
        );

    let ipv6 = groups.get_group(0).unwrap();
    let ipv4 = groups.get_group(1).unwrap();

    merge(vec![tcp_ipv6_nf(ipv6), tcp_ipv4_nf(ipv4)]).compose()
}
