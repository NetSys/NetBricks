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
            println!(
                "hdr {} next_header {} offset {}",
                hdr,
                hdr.next_header(),
                hdr.offset()
            );
            println!(
                "payload: {:x} {:x} {:x} {:x}",
                payload[0], payload[1], payload[2], payload[3]
            );
            let (src, dst) = (flow.src_port, flow.dst_port);
            println!("Src {} dst {}", src, dst);
        })
        .parse::<TcpHeader<Ipv6Header>>()
        .map(box |pkt| {
            println!("TCP header {}", pkt.get_header());
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
            println!("hdr {} offset {}", hdr, hdr.offset());
            println!(
                "payload: {:x} {:x} {:x} {:x}",
                payload[0], payload[1], payload[2], payload[3]
            );
            let (src, dst) = (flow.src_port, flow.dst_port);
            println!("Src {} dst {}", src, dst);
        })
        .parse::<TcpHeader<Ipv4Header>>()
        .map(box |pkt| {
            println!("TCP header {}", pkt.get_header());
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
            println!("hdr {}", pkt.get_header());
            let payload = pkt.get_payload();
            print!("Payload: ");
            for p in payload {
                print!("{:x} ", p);
            }
            println!("");
        })
        .filter(box |pkt| match pkt.get_header().etype() {
            Some(mac::EtherType::IPv4) => true,
            Some(mac::EtherType::IPv6) => true,
            _ => false,
        })
        .group_by(
            2,
            box |pkt| match pkt.get_header().etype().unwrap() {
                mac::EtherType::IPv4 => 1,
                _ => 0,
            },
            sched,
        );

    let ipv6 = groups.get_group(0).unwrap();
    let ipv4 = groups.get_group(1).unwrap();

    merge(vec![tcp_ipv6_nf(ipv6), tcp_ipv4_nf(ipv4)]).compose()
}
