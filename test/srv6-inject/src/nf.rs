use colored::*;
use generic_array::typenum::*;
use generic_array::GenericArray;
use netbricks::headers::*;
use netbricks::interface::*;
use netbricks::operators::*;
use netbricks::scheduler::Scheduler;
use netbricks::utils::*;
use std::default::Default;
use std::net::Ipv6Addr;
use std::str::FromStr;
use CACHE;

#[allow(dead_code)]
enum NewSegmentsAction {
    Prepend,
    Append,
    Overwrite,
}

macro_rules! srh_insert_packet_gen {
    ($segments:expr, $pkt:expr, $meta:expr, $prev_hdr:ty, [$($len:expr => $ty:ty),*]) => {
        match $segments.len() {
            $(
                $len => {
                    let (next_header, payload_len, _, _, _) = $meta;

                    let mut srh = <SegmentRoutingHeader<$prev_hdr, $ty>>::new(
                        *GenericArray::<_, $ty>::from_slice(&$segments[..]),
                    );

                    srh.set_segments_left(1);
                    srh.ext_header.set_next_header(next_header);

                    if let Ok(()) = $pkt.insert_header(&srh) {
                        let mut v6h = $pkt.get_mut_header();
                        v6h.set_next_header(NextHeader::Routing);
                        v6h.set_dst(Segment::from($segments[$len - 1]));
                        v6h.set_payload_len(payload_len + srh.offset() as u16);
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

macro_rules! srh_swap_packet_gen {
    ($segments:expr, $pkt:expr, $prev_hdr:ty, [$($len:expr => $ty:ty),*]) => {
        match $segments.len() {
            $(
                $len => {
                    let mut srh = <SegmentRoutingHeader<$prev_hdr, $ty>>::new_from_tuple(
                        ($pkt.get_header().next_header(),
                         $pkt.get_header().segments_left(),
                         $pkt.get_header().flags(),
                         $pkt.get_header().tag()),
                        *GenericArray::<_, $ty>::from_slice(&$segments[..]),
                    );

                    srh.set_segments_left(1);

                    if let Ok(swap_diff) = $pkt.swap_header::<SegmentRoutingHeader<$prev_hdr, $ty>>(&srh) {
                        return Some((swap_diff, Segment::from($segments[$len - 1])));
                    } else {
                        return None
                    }
                }
            )*
                _ => {
                    None
                }
        };
    };
}

#[derive(Debug)]
struct MetaDataz {
    next_header: NextHeader,
    payload_len: u16,
    flow: FlowV6,
    payload_diff: i8,
    segment_dst: Segment,
}

impl Default for MetaDataz {
    fn default() -> MetaDataz {
        MetaDataz {
            next_header: NextHeader::NoNextHeader,
            payload_len: 0,
            flow: FlowV6::default(),
            payload_diff: 0,
            segment_dst: Ipv6Addr::unspecified(),
        }
    }
}

#[inline]
fn parse_meta<'a>(meta: &MetaDataz) -> (NextHeader, u16, FlowV6, i8, Segment) {
    println!("Parsing MetaDATAZ {}", format!("{:?}", meta).green());
    (
        meta.next_header,
        meta.payload_len,
        meta.flow,
        meta.payload_diff,
        meta.segment_dst,
    )
}

fn srh_into_packet(pkt: &mut Packet<Ipv6Header, MetaDataz>) -> () {
    let seg0 = Ipv6Addr::from_str("fe80::4").unwrap();
    let seg1 = Ipv6Addr::from_str("1ce:c01d:bee2:15:a5:900d:a5:11fe").unwrap();
    let segvec = vec![seg0, seg1];
    srh_insert_packet_gen!(segvec,
                         pkt,
                         parse_meta(pkt.read_metadata()),
                         Ipv6Header,
                         [1 => U1, 2=> U2, 3 => U3, 4 => U4, 5 => U5, 6 => U6, 7 => U7, 8 => U8, 9 => U9, 10 => U10, 11 => U11, 12 => U12])
}

fn srh_change_packet(
    pkt: &mut Packet<SRH<Ipv6Header>, MetaDataz>,
    seg_action: NewSegmentsAction,
) -> Option<(isize, Segment)> {
    let seg1 = Ipv6Addr::from_str("fe80::a").unwrap();
    let mut segvec = vec![seg1];

    match seg_action {
        NewSegmentsAction::Append => {
            segvec.splice(0..0, pkt.get_header().segments().unwrap().iter().cloned());
            ()
        }
        NewSegmentsAction::Prepend => {
            segvec.extend_from_slice(pkt.get_header().segments().unwrap())
        }
        NewSegmentsAction::Overwrite => (),
    }

    srh_swap_packet_gen!(segvec,
                         pkt,
                         Ipv6Header,
                         [1 => U1, 2=> U2, 3 => U3, 4 => U4, 5 => U5, 6 => U6, 7 => U7, 8 => U8, 9 => U9, 10 => U10, 11 => U11, 12 => U12])
}

#[inline]
fn tcp_sr_nf<T: 'static + Batch<Header = Ipv6Header>>(parent: T) -> CompositionBatch {
    parent
        .map(box |pkt| {
            println!("V6-old-hdr {}", format!("{}", pkt.get_header()).yellow());
        })
        .metadata(box |pkt| {
            let v6h = pkt.get_header();
            let flow = v6h.flow().unwrap();

            MetaDataz {
                payload_len: v6h.payload_len(),
                flow: FlowV6 {
                    src_port: flow.src_port,
                    dst_port: flow.dst_port,
                    proto: flow.proto,
                    ..Default::default()
                },
                ..Default::default()
            }
        })
        .parse::<SRH<Ipv6Header>>()
        .transform(box |pkt| {
            if let Some((payload_diff, segment_dst)) = srh_change_packet(pkt, NewSegmentsAction::Prepend) {
                let prev_payload_len = pkt.read_metadata().payload_len;
                let flow  = pkt.read_metadata().flow;

                pkt.write_metadata({
                    &MetaDataz {
                        payload_len: prev_payload_len,
                        payload_diff: payload_diff as i8,
                        segment_dst: segment_dst,
                        flow: flow,
                        ..Default::default()
                    }
                }).unwrap();
            }
        })
        .parse::<TcpHeader<SRH<Ipv6Header>>>()
        .map(box |pkt| {
            println!("TCP header {}", format!("{}", pkt.get_header()).yellow());
        })
        // Using reset instead of deparse b/c .deparse() is currently busted wrt to how operations work in nb.
        .reset()
        .parse::<MacHeader>()
        .parse::<Ipv6Header>()
        .metadata(box |pkt| {
            // Bring back metadata for regular read.
            MetaDataz {
                payload_len: pkt.emit_metadata::<MetaDataz>().payload_len,
                flow: pkt.emit_metadata::<MetaDataz>().flow,
                payload_diff: pkt.emit_metadata::<MetaDataz>().payload_diff,
                segment_dst: pkt.emit_metadata::<MetaDataz>().segment_dst,
                ..Default::default()
            }
        })
        .transform(box |pkt| {
            let (_, prev_payload_len, _, payload_diff, segment_dst) =
                parse_meta(pkt.read_metadata());

            let v6h = pkt.get_mut_header();
            v6h.set_dst(segment_dst);
            v6h.set_payload_len((prev_payload_len as i8 + payload_diff) as u16);
        })
        .map(box |pkt| {
            println!("V6-updated-hdr {}", format!("{}", pkt.get_header()).yellow());
        })
        .parse::<SRH<Ipv6Header>>()
        .map(box |pkt| {
            let cache = &mut *CACHE.write().unwrap();
            cache.entry(flow_hash(&Flows::V6(pkt.read_metadata().flow))).or_insert(
                pkt.get_header().segments().unwrap().to_vec()
            );
            println!("SR-hdr {}", format!("{}", pkt.get_header()).yellow());
        })
        .map(box |pkt| {
            let cache = CACHE.read().unwrap();
            println!("Cache Cache Flow Hash: {}",
                     format!("{:?}", *cache.get(&flow_hash(&Flows::V6(pkt.read_metadata().flow)))
                             .unwrap()).cyan().underline());
        })
        .parse::<TcpHeader<SRH<Ipv6Header>>>()
        .map(box |pkt| {
            println!("TCP header {}", format!("{}", pkt.get_header()).yellow());
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
                flow: v6h.flow().unwrap(),
                ..Default::default()
            }
        })
        .transform(box |pkt| {
            srh_into_packet(pkt);
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
