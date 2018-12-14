use colored::*;
use netbricks::headers::*;
use netbricks::operators::*;
use netbricks::utils::FlowV6;
use std::collections::HashMap;
use std::default::Default;
use std::net::Ipv6Addr;
use std::str::FromStr;

struct Meta {
    v6flow: FlowV6,
    init_checksum: u16,
    new_dst: Ipv6Addr,
}

impl Default for Meta {
    fn default() -> Meta {
        Meta {
            v6flow: FlowV6::default(),
            init_checksum: 0,
            new_dst: Ipv6Addr::UNSPECIFIED,
        }
    }
}

#[inline]
fn tcp_ipv6_nf<T: 'static + Batch<Header = MacHeader>>(parent: T) -> CompositionBatch {
    println!("{}",
             format!("Tests Fully Computed Checksum vs Incrementally Computed Checksums and RoundTrip Checksums").white());

    let map: HashMap<Ipv6Addr, &str> = [
        (Ipv6Addr::from_str("fdaa:0:2:c::11").unwrap(), "F6AF"),
        (Ipv6Addr::from_str("fdaa:0:2:c::12").unwrap(), "F6AE"),
        (Ipv6Addr::from_str("fdaa:0:1::13:0:0").unwrap(), "F0CA"),
        (
            Ipv6Addr::from_str("2001:558:feed:10:0:2::").unwrap(),
            "6A61",
        ),
    ]
    .iter()
    .cloned()
    .collect();
    parent
        .parse::<Ipv6Header>()
        .metadata(box |pkt| Meta {
            v6flow: pkt.get_header().flow().unwrap(),
            new_dst: Ipv6Addr::from_str("2001:db8:85a3::1").unwrap(),
            ..Default::default()
        })
        .parse::<TcpHeader<Ipv6Header>>()
        .metadata_mut(box |pkt| Meta {
            v6flow: pkt.read_metadata().v6flow,
            new_dst: pkt.read_metadata().new_dst,
            init_checksum: pkt.get_header().checksum(),
        })
        .transform(box |pkt| {
            let init_checksum = pkt.read_metadata().init_checksum;
            let src = pkt.read_metadata().v6flow.src_ip;
            let dst = pkt.read_metadata().v6flow.dst_ip;

            println!(
                "{}",
                format!(
                    "   Original CheckSum: {:X?} | Src: {} | Dst: {}",
                    init_checksum, src, dst
                )
                .purple()
            );

            {
                let segment_length = pkt.segment_length();
                let tcph = pkt.get_mut_header();
                tcph.set_checksum(0);
                tcph.update_checksum(segment_length, src, dst);
            }

            let computed_checksum = pkt.get_header().checksum();

            println!(
                "{}\n",
                format!(
                    "Re-Computed CheckSum: {:X?} | Src: {} | Dst: {}",
                    computed_checksum, src, dst
                )
                .purple()
            );

            // Assert that we arrived at the same checksum we removed
            assert_eq!(
                format!("{:X?}", init_checksum),
                format!("{:X?}", computed_checksum)
            );
        })
        .reset()
        .parse::<MacHeader>()
        .parse::<Ipv6Header>()
        .transform(box |pkt| {
            let new_dst = pkt.emit_metadata::<Meta>().new_dst;
            pkt.get_mut_header().set_dst(new_dst);
            assert_eq!(pkt.get_header().dst(), new_dst);
        })
        .parse::<TcpHeader<Ipv6Header>>()
        .transform(box move |pkt| {
            let current_checksum = pkt.get_header().checksum();
            let src = pkt.emit_metadata::<Meta>().v6flow.src_ip;
            let old_dst = pkt.emit_metadata::<Meta>().v6flow.dst_ip;
            let new_dst = pkt.emit_metadata::<Meta>().new_dst;

            println!(
                "{}",
                format!(
                    "      Previous CheckSum: {:X?} | Old Dst: {} | New Dst: {}",
                    current_checksum, old_dst, new_dst
                )
                .purple()
            );

            let segment_length = pkt.segment_length();
            let tcph = pkt.get_mut_header();

            // Check that we're returning an actual, overflow-checked new checksum
            assert!(tcph.update_checksum_incremental(old_dst, new_dst).is_some());
            // Compare to WireShark Validation
            assert_eq!(format!("{:X?}", tcph.checksum()), *map.get(&src).unwrap());

            let incremented_checksum = tcph.checksum();
            tcph.update_checksum(segment_length, src, new_dst);

            // Test Incremented vs Fully Computed Checksums
            assert_eq!(
                format!("{:X?}", incremented_checksum),
                format!("{:X?}", tcph.checksum())
            );

            // Assert that the change to dst, effects the checksum calculation
            assert_ne!(current_checksum, incremented_checksum);

            println!(
                "{}\n",
                format!(
                    "Newly Computed CheckSum: {:X?} | Old Dst: {} | New Dst: {}",
                    incremented_checksum, old_dst, new_dst
                )
                .green()
            );
        })
        .reset()
        .parse::<MacHeader>()
        .parse::<Ipv6Header>()
        .transform(box |pkt| {
            let new_old_dst = pkt.emit_metadata::<Meta>().v6flow.dst_ip;
            pkt.get_mut_header().set_dst(new_old_dst);
            assert_eq!(pkt.get_header().dst(), new_old_dst);
        })
        .parse::<TcpHeader<Ipv6Header>>()
        .transform(box move |pkt| {
            let init_checksum = pkt.emit_metadata::<Meta>().init_checksum;
            let current_checksum = pkt.get_header().checksum();
            let src = pkt.emit_metadata::<Meta>().v6flow.src_ip;
            let old_dst = pkt.emit_metadata::<Meta>().new_dst;
            let new_old_dst = pkt.emit_metadata::<Meta>().v6flow.dst_ip;

            println!(
                "{}",
                format!(
                    "          Previous CheckSum: {:X?} | Old Dst: {} | New Dst: {}",
                    current_checksum, old_dst, new_old_dst
                )
                .green()
            );

            let segment_length = pkt.segment_length();
            let tcph = pkt.get_mut_header();
            // Check that we're returning an actual, overflow-checked new checksum
            assert!(tcph
                .update_checksum_incremental(old_dst, new_old_dst)
                .is_some());
            assert_eq!(
                format!("{:X?}", tcph.checksum()),
                format!("{:X?}", init_checksum)
            );

            let incremented_checksum = tcph.checksum();
            tcph.update_checksum(segment_length, src, new_old_dst);

            // Test Incremented vs Fully Computed Checksums
            assert_eq!(
                format!("{:X?}", incremented_checksum),
                format!("{:X?}", tcph.checksum())
            );

            // Assert that the change to dst, effects the checksum calculation
            assert_ne!(current_checksum, incremented_checksum);

            println!(
                "{}\n",
                format!(
                    "Roundtrip Computed CheckSum: {:X?} | Old Dst: {} | New Dst: {}",
                    incremented_checksum, old_dst, new_old_dst
                )
                .purple()
            );
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
