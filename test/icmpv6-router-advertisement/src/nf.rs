use colored::*;
use netbricks::headers::*;
use netbricks::operators::*;

pub fn icmp_nf<T: 'static + Batch<Header = NullHeader>>(parent: T) -> CompositionBatch {
    let pipeline = parent
        .parse::<MacHeader>()
        .filter(box |pkt| match pkt.get_header().etype() {
            Some(EtherType::IPv6) => true,
            _ => false,
        });

    icmp_v6_router_advertisementertisement_nf(pipeline)
}

#[inline]
fn icmp_v6_router_advertisementertisement_nf<T: 'static + Batch<Header = MacHeader>>(
    parent: T,
) -> CompositionBatch {
    println!(
        "{}",
        format!("Tests ICMPv6 messages for msg_type, code and checksum").white()
    );
    parent
        .parse::<Ipv6Header>()
        .parse::<Icmpv6RouterAdvertisement<Ipv6Header>>()
        .transform(box |pkt| {
            let router_advertisement = pkt.get_mut_header();
            println!(
                "{}",
                format!(
                    "   Msg Type: {:X?} | Code: {} | Checksum: {:X?}",
                    router_advertisement.msg_type().unwrap(),
                    router_advertisement.code(),
                    router_advertisement.checksum()
                ).purple()
            );

            assert_eq!(
                format!("{:X?}", router_advertisement.msg_type().unwrap()),
                format!("{:X?}", IcmpMessageType::RouterAdvertisement)
            );
            assert_eq!(
                format!("{:X?}", router_advertisement.code()),
                format!("{:X?}", 0)
            );
            assert_eq!(
                format!("{:X?}", router_advertisement.checksum()),
                format!("{:X?}", 0xbfbc)
            );
            assert_eq!(
                format!("{:X?}", router_advertisement.current_hop_limit()),
                format!("{:X?}", 64)
            );
            assert_eq!(
                format!("{:X?}", router_advertisement.managed_addr_cfg()),
                format!("{:X?}", false)
            );
            assert_eq!(
                format!("{:X?}", router_advertisement.other_cfg()),
                format!("{:X?}", true)
            );
            assert_eq!(
                format!("{:X?}", router_advertisement.router_lifetime()),
                format!("{:X?}", 1800)
            );
            assert_eq!(
                format!("{:X?}", router_advertisement.reachable_time()),
                format!("{:X?}", 600)
            );
            assert_eq!(
                format!("{:X?}", router_advertisement.retrans_timer()),
                format!("{:X?}", 500)
            );
        })
        .compose()
}
