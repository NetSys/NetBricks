use colored::*;
use netbricks::headers::*;
use netbricks::operators::*;
use std::collections::HashMap;
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
            msg_type: None,
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
            ..Default::default()
        })
        .parse::<IcmpV6Header<Ipv6Header>>()
        .metadata_mut(box |pkt| Meta {
            msg_type: IcmpMessageType.from_u8(134),
            code: pkt.read_metadata().code,
            checksum: pkt.read_metadata().checksum,
            src: Ipv6Addr.from_str("fe80::d4f0:45ff:fe0c:664b").unwrap(),
            dst: IpV6Addr.from_str("ff02::1").unwrap()
        })
        .transform(box |pkt| {
            let init_checksum = pkt.read_metadata().checksum;

            println!(
                "{}",
                format!(
                    "   Original CheckSum: {:X?} | Src: {} | Dst: {}",
                    init_checksum, src, dst
                ).purple()
            );

            {
                let segment_length = pkt.segment_length();
                let tcph = pkt.get_mut_header();
                tcph.set_checksum(0);
            }

            let computed_checksum = pkt.get_header().checksum();

            println!(
                "{}\n",
                format!(
                    "Re-Computed CheckSum: {:X?} | Src: {} | Dst: {}",
                    computed_checksum, src, dst
                ).purple()
            );

            // Assert that we arrived at the same checksum we removed
            assert_eq!(
                format!("{:X?}", init_checksum),
                format!("{:X?}", computed_checksum)
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
