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

    icmp_v6_neighbor_solicitation_nf(pipeline)
}

#[inline]
fn icmp_v6_neighbor_solicitation_nf<T: 'static + Batch<Header = MacHeader>>(
    parent: T,
) -> CompositionBatch {
    println!(
        "{}",
        format!("Tests ICMPv6 messages for msg_type, code and checksum").white()
    );
    parent
        .parse::<Ipv6Header>()
        .parse::<Icmpv6NeighborSolicitation<Ipv6Header>>()
        .transform(box |pkt| {
            let neighbor_solicitation = pkt.get_mut_header();
            println!(
                "{}",
                format!(
                    "   Msg Type: {:X?} | Code: {} | Checksum: {:X?}",
                    neighbor_solicitation.msg_type().unwrap(),
                    neighbor_solicitation.code(),
                    neighbor_solicitation.checksum()
                ).purple()
            );

            assert_eq!(
                format!("{:X?}", neighbor_solicitation.msg_type().unwrap()),
                format!("{:X?}", IcmpMessageType::NeighborSolicitation)
            );
            assert_eq!(
                format!("{:X?}", neighbor_solicitation.code()),
                format!("{:X?}", 0)
            );
            assert_eq!(
                format!("{:X?}", neighbor_solicitation.checksum()),
                format!("{:X?}", 0xf50c)
            );
            assert_eq!(
                format!("{:X?}", neighbor_solicitation.reserved_flags()),
                format!("{:X?}", 0)
            );

            let expected_target_addr = Ipv6Addr::from_str("fe80::c001:2ff:fe40:0").unwrap();
            assert_eq!(
                format!("{:X?}", neighbor_solicitation.target_addr()),
                format!("{:X?}", expected_target_addr)
            );
        })
        .compose()
}
