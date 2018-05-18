use colored::*;
use generic_array::typenum::*;
use generic_array::GenericArray;
use netbricks::headers::*;
use netbricks::interface::*;
use netbricks::operators::*;
use netbricks::scheduler::Scheduler;
use std::net::Ipv6Addr;
use std::str::FromStr;

macro_rules! srh_into_packet_gen {
    ($segments:expr, $pkt:expr, $meta:expr, [$($len:expr => $ty:ty),*]) => {
        match $segments.len() {
            $(
                $len => {
                    let mut srh = <SegmentRoutingHeader<Ipv6Header, $ty>>::new(
                        *GenericArray::<_, $ty>::from_slice(&$segments[..]),
                    );

                    let (nh, payload_len) = $meta;

                    srh.set_segments_left(1);
                    srh.set_last_entry(1);
                    srh.ext_header.set_next_header(nh);

                    if let Ok(()) = $pkt.insert_header(&srh) {
                        let mut v6pkt = $pkt.get_mut_header();
                        v6pkt.set_next_header(NextHeader::Routing);
                        v6pkt.set_dst(Segment::from($segments[$len - 1]));
                        v6pkt.set_payload_len(payload_len + (srh.offset() as u16));
                    } else {
                        ()
                    }
                }
            )*
                _ => {
                    ()
                }
        };
    };
}

#[derive(Debug)]
struct MetaDataz {
    next_header: NextHeader,
    payload_len: u16,
}

#[inline]
fn parse_meta(meta: &MetaDataz) -> (NextHeader, u16) {
    println!("Parsing MetaDATAZ {}", format!("{:?}", meta).green());
    (meta.next_header, meta.payload_len)
}

fn srh_into_packet(pkt: &mut Packet<Ipv6Header, MetaDataz>, meta: (NextHeader, u16)) -> () {
    let seg0 = Ipv6Addr::from_str("fe80::4").unwrap();
    let seg1 = Ipv6Addr::from_str("1ce:c01d:bee2:15:a5:900d:a5:11fe").unwrap();
    let segvec = vec![seg0, seg1];
    srh_into_packet_gen!(segvec,
                         pkt,
                         meta,
                         [1 => U1, 2=> U2, 3 => U3, 4 => U4, 5 => U5, 6 => U6, 7 => U7, 8 => U8, 9 => U9, 10 => U10, 11 => U11, 12 => U12]);
}

#[inline]
fn tcp_sr_nf<T: 'static + Batch<Header = Ipv6Header>>(parent: T) -> CompositionBatch {
    parent
        .parse::<SRH<Ipv6Header>>()
        .map(box |pkt| {
            println!("SR-hdr {}", format!("{}", pkt.get_header()).red());
        })
        .parse::<TcpHeader<SRH<Ipv6Header>>>()
        .map(box |pkt| {
            println!("TCP header {}", format!("{}", pkt.get_header()).red());
        })
        .compose()
}

#[inline]
fn tcp_sr_inject_nf<T: 'static + Batch<Header = Ipv6Header>>(parent: T) -> CompositionBatch {
    parent
        .metadata(box |pkt| {
            let v6h = pkt.get_header();

            MetaDataz {
                next_header: v6h.next_header().unwrap(),
                payload_len: v6h.payload_len(),
            }
        })
        .transform(box |pkt| {
            let meta = parse_meta(pkt.read_metadata());
            srh_into_packet(pkt, meta);
        })
        .filter(box |pkt| match pkt.get_header().next_header() {
            Some(NextHeader::Routing) => true,
            _ => false,
        })
        .parse::<SRH<Ipv6Header>>()
        .map(box |pkt| {
            println!("SR-hdr {}", format!("{}", pkt.get_header()).green());
        })
        .parse::<TcpHeader<SRH<Ipv6Header>>>()
        .map(box |pkt| {
            println!("TCP header {}", format!("{}", pkt.get_header()).green());
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

    let srv6_inject = groups.get_group(0).unwrap();
    let srv6 = groups.get_group(1).unwrap();

    merge(vec![tcp_sr_nf(srv6), tcp_sr_inject_nf(srv6_inject)]).compose()
}
