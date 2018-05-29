use netbricks::config::Atom;
use netbricks::headers::*;
use netbricks::operators::*;
use netbricks::scheduler::*;
use Bar;

#[inline]
fn tcp_ipv6_nf<T: 'static + Batch<Header = Ipv6Header>>(parent: T) -> CompositionBatch {
    parent.parse::<TcpHeader<Ipv6Header>>().compose()
}

#[inline]
fn tcp_sr_nf<T: 'static + Batch<Header = Ipv6Header>>(parent: T) -> CompositionBatch {
    parent
        .parse::<SRH<Ipv6Header>>()
        .map(box |pkt| {
            info!("SR-hdr {}", pkt.get_header());
        })
        .parse::<TcpHeader<SRH<Ipv6Header>>>()
        .compose()
}

pub fn nf<T: 'static + Batch<Header = NullHeader>, S: Scheduler + Sized>(
    parent: T,
    sched: &mut S,
    configuration: &'static Atom<Bar>,
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
        .map(box move |_pkt| {
            info!(
                "Settings/Configuration Static State Val: {:?}",
                configuration.get()
            );
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
