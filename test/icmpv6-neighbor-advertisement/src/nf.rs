use colored::*;
use netbricks::headers::*;
use netbricks::operators::*;
use std::net::Ipv6Addr;
use std::str::FromStr;

pub fn icmp_nf<T: 'static + Batch<Header = NullHeader>>(parent: T) -> CompositionBatch {
    let pipeline = parent
        .parse::<MacHeader>()
        .filter(box |pkt| match pkt.get_header().etype() {
            Some(EtherType::IPv6) => true,
            _ => false,
        });

    icmp_v6_neighbor_advertisement_nf(pipeline)
}

#[inline]
fn icmp_v6_neighbor_advertisement_nf<T: 'static + Batch<Header = MacHeader>>(
    parent: T,
) -> CompositionBatch {
    println!(
        "{}",
        format!("Tests ICMPv6 messages for msg_type, code and checksum").white()
    );
    parent
        .parse::<Ipv6Header>()
        .parse::<Icmpv6NeighborAdvertisement<Ipv6Header>>()
        .transform(box |pkt| {
            let neighbor_advertisement = pkt.get_mut_header();
            println!(
                "{}",
                format!(
                    "   Msg Type: {:X?} | Code: {} | Checksum: {:X?}",
                    neighbor_advertisement.msg_type().unwrap(),
                    neighbor_advertisement.code(),
                    neighbor_advertisement.checksum()
                )
                .purple()
            );

            assert_eq!(
                format!("{:X?}", neighbor_advertisement.msg_type().unwrap()),
                format!("{:X?}", IcmpMessageType::NeighborAdvertisement)
            );
            assert_eq!(
                format!("{:X?}", neighbor_advertisement.code()),
                format!("{:X?}", 0)
            );
            assert_eq!(
                format!("{:X?}", neighbor_advertisement.checksum()),
                format!("{:X?}", 0x0d2b)
            );

            let expected_target_addr = Ipv6Addr::from_str("fe80::c002:3ff:fee4:0").unwrap();

            assert_eq!(
                format!("{:X?}", neighbor_advertisement.target_addr()),
                format!("{:X?}", expected_target_addr)
            );
            assert_eq!(
                format!("{:X?}", neighbor_advertisement.router_flag()),
                format!("{:X?}", false)
            );
            assert_eq!(
                format!("{:X?}", neighbor_advertisement.solicitated_flag()),
                format!("{:X?}", true)
            );
            assert_eq!(
                format!("{:X?}", neighbor_advertisement.override_flag()),
                format!("{:X?}", true)
            );
        })
        .compose()
}
