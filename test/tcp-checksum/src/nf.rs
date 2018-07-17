use colored::*;
use netbricks::headers::*;
use netbricks::operators::*;

#[inline]
fn tcp_ipv6_nf<T: 'static + Batch<Header = MacHeader>>(parent: T) -> CompositionBatch {
    parent
        .parse::<Ipv6Header>()
        .metadata(box |pkt| pkt.get_header().flow().unwrap())
        .parse::<TcpHeader<Ipv6Header>>()
        .transform(box |pkt| {
            let init_checksum = pkt.get_header().checksum();
            let src = pkt.read_metadata().src_ip;
            let dst = pkt.read_metadata().dst_ip;

            println!(
                "{}",
                format!(
                    "CheckSum: {} ~ {:x?} | Src: {} | Dst: {}",
                    init_checksum, init_checksum, src, dst
                ).cyan()
            );

            {
                let segment_length = pkt.segment_length();
                let v6h = pkt.get_mut_header();
                v6h.update_checksum(segment_length, src, dst);
            }

            let computed_checksum = pkt.get_header().checksum();

            println!(
                "{}",
                format!(
                    "CheckSum: {} ~ {:x?} | Src: {} | Dst: {}",
                    computed_checksum, computed_checksum, src, dst
                ).white()
            );

            assert_eq!(init_checksum, computed_checksum)
        })
        .compose()
}

pub fn tcp_nf<T: 'static + Batch<Header = NullHeader>>(parent: T) -> CompositionBatch {
    let pipeline = parent
        .parse::<MacHeader>()
        .filter(box |pkt| match pkt.get_header().etype() {
            Some(EtherType::IPv6) => true,
            _ => false,
        });

    tcp_ipv6_nf(pipeline)
}
