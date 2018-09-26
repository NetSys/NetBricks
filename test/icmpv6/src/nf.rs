use colored::*;
use netbricks::headers::*;
use netbricks::operators::*;
use std::default::Default;
use std::net::Ipv6Addr;
use std::str::FromStr;

struct Meta {
    msg_type: IcmpMessageType,
    code: u8,
    checksum: u16,
    src: Ipv6Addr,
    dst: Ipv6Addr
}

impl Default for Meta {
    fn default() -> Meta {
        Meta {
            msg_type: IcmpMessageType::NeighborAdvertisement,
            code: 0,
            checksum: 0,
            src: Ipv6Addr::unspecified(),
            dst: Ipv6Addr::unspecified()
        }
    }
}

#[inline]
fn icmp_v6_nf<T: 'static + Batch<Header = MacHeader>>(parent: T) -> CompositionBatch {
    println!("{}",
             format!("Tests Fully Computed Checksum vs Incrementally Computed Checksums and RoundTrip Checksums").white());
    parent
        .parse::<Ipv6Header>()
        .metadata(box |pkt| Meta {
            src: Ipv6Addr::from_str("fe80::d4f0:45ff:fe0c:664b").unwrap(),
            dst: Ipv6Addr::from_str("ff02::1").unwrap(),
            ..Default::default()
        })
        .parse::<IcmpV6Header<Ipv6Header>>()
        .metadata(box | pkt | Meta {
            msg_type: pkt.get_header().msg_type().unwrap(),
            code: pkt.get_header().code(),
            checksum: pkt.get_header().checksum(),
            src: Ipv6Addr::from_str("fe80::d4f0:45ff:fe0c:664b").unwrap(),
            dst: Ipv6Addr::from_str("ff02::1").unwrap(),
            ..Default::default()

        })
        .transform(box |pkt| {
            let src = pkt.read_metadata().src;
            let dst = pkt.read_metadata().dst;

            let msg_type = &pkt.read_metadata().msg_type;
            let code = pkt.read_metadata().code;
            let checksum = pkt.read_metadata().checksum;

            println!(
                "{}",
                format!(
                    " Src Ip {:X?} |  Dst Ip {:X?} | Msg Type: {:X?} | Code: {:X?} | Checksum: {:X?}",
                    src, dst, msg_type, code, checksum
                ).purple()
            );

            assert_eq!(
                format!("{:X?}", msg_type),
                format!("{:X?}", IcmpMessageType::NeighborAdvertisement)
            );
            assert_eq!(
                format!("{:X?}", code),
                format!("{:X?}", 0)
            );
            assert_eq!(
                format!("{:X?}", checksum),
                format!("{:X?}", 0xf50c)
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
