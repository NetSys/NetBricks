use netbricks::headers::*;
use netbricks::operators::*;
use netbricks::scheduler::Scheduler;

#[inline]
fn tcp_ipv6_nf<T: 'static + Batch<Header = Ipv6Header>>(parent: T) -> CompositionBatch {
    parent
        .map(box |pkt| {
            println!("IPv6-normcore hdr {}", pkt.get_header());
        })
        .parse::<TcpHeader<Ipv6Header>>()
        .map(box |pkt| {
            println!("TCP header {}", pkt.get_header());
        })
        .compose()
}

fn tcp_sr_nf<T: 'static + Batch<Header = Ipv6Header>>(parent: T) -> CompositionBatch {
    parent
        .parse::<SegmentRoutingHeader<Ipv6Header>>()
        .map(box |pkt| {
            println!("SR-hdr {}", pkt.get_header());
        })
        .parse::<TcpHeader<SegmentRoutingHeader<Ipv6Header>>>()
        .map(box |pkt| {
            println!("TCP header {}", pkt.get_header());
        })
        .compose()
}

pub fn nf<T: 'static + Batch<Header = NullHeader>, S: Scheduler + Sized>(
    parent: T,
    sched: &mut S,
) -> CompositionBatch {
    let mut groups = parent
        .parse::<MacHeader>()
        .filter(box |pkt| match pkt.get_header().etype() {
            Some(EtherType::IPv6) => true,
            _ => false,
        })
        .parse::<Ipv6Header>()
        .filter(box |pkt| match pkt.get_header().next_header() {
            Some(NextHeader::Routing) => true,
            Some(NextHeader::Tcp) => true,
            _ => false,
        })
        .group_by(
            2,
            box |pkt| match pkt.get_header().next_header().unwrap() {
                NextHeader::Routing => 1,
                _ => 0,
            },
            sched,
        );

    let ipv6only = groups.get_group(0).unwrap();
    let srv6 = groups.get_group(1).unwrap();

    merge(vec![tcp_ipv6_nf(ipv6only), tcp_sr_nf(srv6)]).compose()
}
