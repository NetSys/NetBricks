use colored::*;
use netbricks::headers::*;
use netbricks::operators::*;
use std::default::Default;

struct Meta {
    msg_type: IcmpMessageType,
    code: u8,
    checksum: u16,
}

impl Default for Meta {
    fn default() -> Meta {
        Meta {
            msg_type: IcmpMessageType::NeighborAdvertisement,
            code: 0,
            checksum: 0,
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
        .parse::<IcmpV6Header<Ipv6Header>>()
        .metadata(box |pkt| Meta {
            msg_type: pkt.get_header().msg_type().unwrap(),
            code: pkt.get_header().code(),
            checksum: pkt.get_header().checksum(),
        })
        .transform(box |pkt| {
            let msg_type = &pkt.read_metadata().msg_type;
            let code = pkt.read_metadata().code;
            let checksum = pkt.read_metadata().checksum;

            println!(
                "{}",
                format!(
                    "   Msg Type: {:X?} | Code: {} | Checksum: {}",
                    msg_type, code, checksum
                )
                .purple()
            );

            assert_eq!(
                format!("{:X?}", msg_type),
                format!("{:X?}", IcmpMessageType::RouterAdvertisement)
            );
            assert_eq!(format!("{:X?}", code), format!("{:X?}", 0));
            assert_eq!(format!("{:X?}", checksum), format!("{:X?}", 0xf50c));
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
