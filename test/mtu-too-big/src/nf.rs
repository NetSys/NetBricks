use netbricks::headers::*;
use netbricks::operators::*;
use netbricks::scheduler::*;

struct Meta {
    newv6: Ipv6Header,
}

pub fn nf<T: 'static + Batch<Header = NullHeader>, S: Scheduler + Sized>(
    parent: T,
    sched: &mut S,
) -> CompositionBatch {
    let mut groups = parent.parse::<MacHeader>().group_by(
        2,
        box |pkt| match pkt.get_payload().len() as u32
            > IPV6_MIN_MTU_CHECK - (MacHeader::size() as u32)
        {
            true => {
                assert_eq!(pkt.get_payload().len() + MacHeader::size(), pkt.data_len());
                0
            }
            _ => 1,
        },
        sched,
    );

    let toobig = groups.get_group(0).unwrap();
    let otherwise = groups.get_group(1).unwrap();

    merge(vec![send_too_big(toobig), pass(otherwise)]).compose()
}

#[inline]
fn send_too_big<T: 'static + Batch<Header = MacHeader>>(parent: T) -> CompositionBatch {
    parent
        .transform(box |pkt| {
            let mach = pkt.get_mut_header();

            // just for test purposes
            let old_src = mach.src.clone();
            let old_dst = mach.dst.clone();

            mach.swap_addresses();

            // Make sure we swap macs to send TOO BIG PACKET back to source
            assert_eq!(mach.src, old_dst);
            assert_eq!(mach.dst, old_src)
        })
        .parse::<Ipv6Header>()
        .transform(box |pkt| {
            assert_eq!(
                pkt.get_header().payload_len() as usize,
                pkt.get_payload().len()
            );
            assert!(pkt.get_header().payload_len() as u32 >= IPV6_MIN_MTU);

            // create new ipv6 header from current header and store in metadata
            let ipv6h_src = pkt.get_header().src();
            let ipv6h_dst = pkt.get_header().dst();

            let mut ipv6h_new = Ipv6Header::new();
            ipv6h_new.set_src(ipv6h_dst);
            ipv6h_new.set_dst(ipv6h_src);
            ipv6h_new.set_next_header(NextHeader::Icmp);

            assert_eq!(ipv6h_new.src(), pkt.get_header().dst());
            assert_eq!(ipv6h_new.dst(), pkt.get_header().src());

            pkt.write_metadata({ &Meta { newv6: ipv6h_new } }).unwrap();
        })
        // We reset and begin process of going from invalid Ipv6 packet into an
        // Icmpv6 - Packet Too Big one.
        //
        // Note: Due to PreviousHeader assoc. type needs, the better bet was to
        // push a new Ipv6 and Icmpv6 Header onto the packet, and let the rest
        // push downward, as we need it as part of the Icmpv6 payload.
        // Then, we trim at the end to get to the allotted set of bytes ~ 1294.
        .reset()
        .parse::<MacHeader>()
        .transform(box |pkt| {
            let ipv6h_new = pkt.emit_metadata::<Meta>().newv6;
            pkt.insert_header(NextHeader::NoNextHeader, &ipv6h_new)
                .unwrap();
        })
        .parse::<Ipv6Header>()
        .transform(box |pkt| {
            // Write IcmpHeader
            let icmpv6 = <Icmpv6PktTooBig<Ipv6Header>>::new();
            // push icmpc6header
            pkt.insert_header(NextHeader::Icmp, &icmpv6).unwrap();

            // Specify/set our v6 payload length, as we trim in the next section.
            pkt.get_mut_header()
                .set_payload_len(IPV6_TOO_BIG_PAYLOAD_LEN);
            assert_eq!(pkt.get_header().payload_len(), 1240)
        })
        .parse::<Icmpv6PktTooBig<Ipv6Header>>()
        .transform(box |pkt| {
            let payload_len = pkt.get_payload().len();
            // Trim Invalid Ipv6 -> End payload until we reach IPV6_MIN_MTU
            pkt.trim_payload_size(
                payload_len
                    - ((IPV6_TOO_BIG_PAYLOAD_LEN as usize) - <Icmpv6PktTooBig<Ipv6Header>>::size()),
            );

            // Assure that we're 1240 - 8 (8 being the icmpv6 pkt too big size)
            assert_eq!(pkt.get_payload().len(), 1232);

            // Generate Checksum for Packet
            // TODO: Is this Offloadable?
            let segment_length = pkt.segment_length(Protocol::Icmp);
            let ipv6h_new = pkt.emit_metadata::<Meta>().newv6;
            let icmpv6_toobig = pkt.get_mut_header();
            icmpv6_toobig.icmp.update_v6_checksum(
                segment_length,
                ipv6h_new.src(),
                ipv6h_new.dst(),
                Protocol::Icmp,
            );
        })
        .compose()
}

#[inline]
fn pass<T: 'static + Batch<Header = MacHeader>>(parent: T) -> CompositionBatch {
    parent.compose()
}
