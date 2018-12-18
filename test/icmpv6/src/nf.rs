use colored::*;
use netbricks::headers::*;
use netbricks::operators::*;
use std::default::Default;
use std::net::Ipv6Addr;

struct Meta {
    src_ip: Ipv6Addr,
    dst_ip: Ipv6Addr,
}

impl Default for Meta {
    fn default() -> Meta {
        Meta {
            src_ip: Ipv6Addr::UNSPECIFIED,
            dst_ip: Ipv6Addr::UNSPECIFIED,
        }
    }
}

#[inline]
fn icmp_v6_nf<T: 'static + Batch<Header = MacHeader>>(parent: T) -> CompositionBatch {
    println!(
        "{}",
        format!("Tests ICMPv6 messages for msg_type, code and checksum").white()
    );
    parent
        .parse::<Ipv6Header>()
        .metadata(box |pkt| Meta {
            src_ip: pkt.get_header().src(),
            dst_ip: pkt.get_header().dst(),
        })
        .parse::<Icmpv6Header<Ipv6Header>>()
        .transform(box |pkt| {
            let segment_length = pkt.segment_length(Protocol::Icmp);
            let src = pkt.read_metadata().src_ip;
            let dst = pkt.read_metadata().dst_ip;

            let icmpv6 = pkt.get_mut_header();
            println!(
                "{}",
                format!(
                    "   Msg Type: {:X?} | Code: {} | Checksum: {:X?}",
                    icmpv6.msg_type().unwrap(),
                    icmpv6.code(),
                    icmpv6.checksum()
                )
                .purple()
            );

            assert_eq!(
                format!("{:X?}", icmpv6.msg_type().unwrap()),
                format!("{:X?}", IcmpMessageType::RouterAdvertisement)
            );
            assert_eq!(format!("{:X?}", icmpv6.code()), format!("{:X?}", 0));
            assert_eq!(
                format!("{:X?}", icmpv6.checksum()),
                format!("{:X?}", 0xf50c)
            );

            let prev_checksum = icmpv6.checksum();
            icmpv6.set_checksum(0);
            icmpv6.update_v6_checksum(segment_length, src, dst, Protocol::Icmp);

            assert_eq!(
                format!("{:X?}", prev_checksum),
                format!("{:X?}", icmpv6.checksum())
            );
        })
        .compose()
}

pub fn icmp_nf<T: 'static + Batch<Header = NullHeader>>(parent: T) -> CompositionBatch {
    let pipeline = parent
        .parse::<MacHeader>()
        .filter(box |pkt| match pkt.get_header().etype() {
            Some(EtherType::IPv6) => true,
            _ => false,
        });

    icmp_v6_nf(pipeline)
}
