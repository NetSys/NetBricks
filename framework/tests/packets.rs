#![feature(tool_attributes)]
extern crate generic_array;
extern crate netbricks;
use generic_array::typenum::*;
use generic_array::GenericArray;
use netbricks::common::EmptyMetadata;
use netbricks::headers::*;
use netbricks::interface::{dpdk, new_packet, Packet};
use std::convert::From;
use std::net::Ipv6Addr;
use std::str::FromStr;
use std::sync::{Once, ONCE_INIT};

static EAL_INIT: Once = ONCE_INIT;

#[rustfmt::skip]
static SRH_BYTES: [u8; 170] = [
    // --- Ethernet header ---
    // Destination MAC
    0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
    // Source MAC
    0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
    // EtherType (IPv6)
    0x86, 0xDD,
    // --- IPv6 Header ---
    // Version, Traffic Class, Flow Label
    0x60, 0x00, 0x00, 0x00,
    // Payload Length
    0x00, 0x74,
    // Next Header (Routing = 43)
    0x2b,
    // Hop Limit
    0x02,
    // Source Address
    0x20, 0x01, 0x0d, 0xb8, 0x85, 0xa3, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
    // Dest Address
    0x20, 0x01, 0x0d, 0xb8, 0x85, 0xa3, 0x00, 0x00, 0x00, 0x00, 0x8a, 0x2e, 0x03, 0x70, 0x73, 0x34,
    // --- SRv6 Header --
    // Next Header (TCP)
    0x06,
    // Hdr Ext Len (3 segments, units of 8 octets or 64 bits)
    0x06,
    // Routing type (SRv6)
    0x04,
    // Segments left
    0x00,
    // Last entry
    0x02,
    // Flags
    0x00,
    // Tag
    0x00, 0x00,
    // Segments: [0] 2001:0db8:85a3:0000:0000:8a2e:0370:7334
    0x20, 0x01, 0x0d, 0xb8, 0x85, 0xa3, 0x00, 0x00, 0x00, 0x00, 0x8a, 0x2e, 0x03, 0x70, 0x73, 0x34,
    // Segments: [1] 2001:0db8:85a3:0000:0000:8a2e:0370:7335
    0x20, 0x01, 0x0d, 0xb8, 0x85, 0xa3, 0x00, 0x00, 0x00, 0x00, 0x8a, 0x2e, 0x03, 0x70, 0x73, 0x35,
    // Segments: [2] 2001:0db8:85a3:0000:0000:8a2e:0370:7333
    0x20, 0x01, 0x0d, 0xb8, 0x85, 0xa3, 0x00, 0x00, 0x00, 0x00, 0x8a, 0x2e, 0x03, 0x70, 0x73, 0x33,
    // --- Tcp header ---
    // Src Port
    0x0d, 0x88,
    // Dst Port
    0x04, 0x00,
    // Sequence number
    0x00, 0x00, 0x00, 0x00,
    // Ack number
    0x00, 0x00, 0x00, 0x00,
    // Flags
    0x50, 0x02,
    // Window
    0x00, 0x0a,
    // Checksum
    0x00, 0x00,
    // Urgent pointer
    0x00, 0x00,
    // Payload
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07,
];

#[rustfmt::skip]
static V6_BYTES: [u8; 62] = [
    // --- Ethernet header ---
    // Destination MAC
    0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
    // Source MAC
    0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
    // EtherType (IPv6)
    0x86, 0xDD,
    // --- IPv6 Header ---
    // Version, Traffic Class, Flow Label
    0x60, 0x00, 0x00, 0x00,
    // Payload Length
    0x00, 0x08,
    // Next Header (UDP = 17)
    0x11,
    // Hop Limit
    0x02,
    // Source Address
    0x20, 0x01, 0x0d, 0xb8, 0x85, 0xa3, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
    // Dest Address
    0x20, 0x01, 0x0d, 0xb8, 0x85, 0xa3, 0x00, 0x00, 0x00, 0x00, 0x8a, 0x2e, 0x03, 0x70, 0x73, 0x34,
    // --- UDP Header ---
    // Src Port
    0x0d, 0x88,
    // Dst Port
    0x04, 0x00,
    // Length
    0x00, 0x08,
    // Checksum
    0x00, 0x00
];

fn setup() {
    EAL_INIT.call_once(|| {
        dpdk::init_system_wl("packet_overlay_tests", 0, &[]);
    });
}

// Acquire a packet buffer for testing header extraction from raw bytes
fn packet_from_bytes(bytes: &[u8]) -> Packet<NullHeader, EmptyMetadata> {
    let mut pkt = new_packet().expect("Could not allocate packet!");
    pkt.increase_payload_size(bytes.len());
    {
        let payload = pkt.get_mut_payload();
        unsafe { bytes.as_ptr().copy_to(payload.as_mut_ptr(), bytes.len()) }
    }
    pkt
}

fn srh_from_bytes() {
    let pkt = packet_from_bytes(&SRH_BYTES);

    // Check Ethernet header
    let epkt = pkt.parse_header::<MacHeader>();
    {
        let eth = epkt.get_header();
        assert_eq!(eth.dst.addr, MacAddress::new(0, 0, 0, 0, 0, 1).addr);
        assert_eq!(eth.src.addr, MacAddress::new(0, 0, 0, 0, 0, 2).addr);
        assert_eq!(eth.etype(), Some(EtherType::IPv6));
    }

    // Check IPv6 header
    let v6pkt = epkt.parse_header::<Ipv6Header>();
    {
        let v6 = v6pkt.get_header();
        let src = Ipv6Addr::from_str("2001:db8:85a3::1").unwrap();
        let dst = Ipv6Addr::from_str("2001:db8:85a3::8a2e:0370:7334").unwrap();
        assert_eq!(v6.version(), 6);
        assert_eq!(v6.traffic_class(), 0);
        assert_eq!(v6.flow_label(), 0);
        assert_eq!(v6.payload_len(), 116);
        assert_eq!(v6.next_header().unwrap(), NextHeader::Routing);
        assert_eq!(v6.hop_limit(), 2);
        assert_eq!(Ipv6Addr::from(v6.src()), src);
        assert_eq!(Ipv6Addr::from(v6.dst()), dst);

        let flow = v6.flow().unwrap();
        assert_eq!(flow.src_ip, src);
        assert_eq!(flow.dst_ip, dst);
        assert_eq!(flow.src_port, 3464);
        assert_eq!(flow.dst_port, 1024);
        assert_eq!(flow.proto, TCP_NXT_HDR);
    }

    // Check SRH
    let srhpkt = v6pkt.parse_header::<SRH<Ipv6Header>>();
    {
        let srh = srhpkt.get_header();
        let seg0 = Ipv6Addr::from_str("2001:db8:85a3::8a2e:0370:7334").unwrap();
        let seg1 = Ipv6Addr::from_str("2001:db8:85a3::8a2e:0370:7335").unwrap();
        let seg2 = Ipv6Addr::from_str("2001:db8:85a3::8a2e:0370:7333").unwrap();
        assert_eq!(srh.next_header().unwrap(), NextHeader::Tcp);
        assert_eq!(srh.hdr_ext_len(), 6);
        assert_eq!(srh.routing_type(), 4);
        assert_eq!(srh.segments_left(), 0);
        assert_eq!(srh.last_entry(), 2);
        assert_eq!(srh.protected(), false);
        assert_eq!(srh.oam(), false);
        assert_eq!(srh.alert(), false);
        assert_eq!(srh.hmac(), false);
        assert_eq!(srh.tag(), 0);
        assert_eq!(srh.segments().unwrap().len(), 3);
        assert_eq!(srh.segments().unwrap()[0], seg0);
        assert_eq!(srh.segments().unwrap()[1], seg1);
        assert_eq!(srh.segments().unwrap()[2], seg2);
    }
}

fn v6_from_bytes() {
    let pkt = packet_from_bytes(&V6_BYTES);

    // Check Ethernet header
    let epkt = pkt.parse_header::<MacHeader>();
    {
        let eth = epkt.get_header();
        assert_eq!(eth.dst.addr, MacAddress::new(0, 0, 0, 0, 0, 1).addr);
        assert_eq!(eth.src.addr, MacAddress::new(0, 0, 0, 0, 0, 2).addr);
        assert_eq!(eth.etype(), Some(EtherType::IPv6));
    }

    // Check IPv6 header
    let v6pkt = epkt.parse_header::<Ipv6Header>();
    {
        let v6 = v6pkt.get_header();
        let src = Ipv6Addr::from_str("2001:db8:85a3::1").unwrap();
        let dst = Ipv6Addr::from_str("2001:db8:85a3::8a2e:0370:7334").unwrap();
        assert_eq!(v6.version(), 6);
        assert_eq!(v6.traffic_class(), 0);
        assert_eq!(v6.flow_label(), 0);
        assert_eq!(v6.payload_len(), 8);
        assert_eq!(v6.next_header().unwrap(), NextHeader::Udp);
        assert_eq!(v6.hop_limit(), 2);
        assert_eq!(Ipv6Addr::from(v6.src()), src);
        assert_eq!(Ipv6Addr::from(v6.dst()), dst);

        let flow = v6.flow().unwrap();
        assert_eq!(flow.src_ip, src);
        assert_eq!(flow.dst_ip, dst);
        assert_eq!(flow.src_port, 3464);
        assert_eq!(flow.dst_port, 1024);
        assert_eq!(flow.proto, UDP_NXT_HDR);
    }
}

fn insert_static_srh_from_bytes() {
    let pkt = packet_from_bytes(&V6_BYTES);
    let epkt = pkt.parse_header::<MacHeader>();
    let mut v6pkt = epkt.parse_header::<Ipv6Header>();
    let mut v6pkt2 = v6pkt.clone();
    let mut v6pkt3 = v6pkt.clone();
    let v6h = v6pkt.get_mut_header();
    {
        assert_eq!(v6h.next_header().unwrap(), NextHeader::Udp);
        v6h.set_next_header(NextHeader::Routing);
        assert_eq!(v6h.next_header().unwrap(), NextHeader::Routing);
    }

    let seg0 = Ipv6Addr::from_str("2001:db8:85a3::8a2e:0370:7335").unwrap();
    let seg1 = Ipv6Addr::from_str("2001:db8:85a3::8a2e:0370:7336").unwrap();
    let segs = vec![seg0, seg1];
    let mut srh =
        <SegmentRoutingHeader<Ipv6Header, U2>>::new(*GenericArray::<_, U2>::from_slice(&segs[..]));
    {
        srh.set_segments_left(0);
        assert_eq!(srh.ext_header.hdr_ext_len(), 4);
        assert_eq!(srh.next_header().unwrap(), NextHeader::NoNextHeader);
        assert_eq!(srh.segments().unwrap().len(), 2);
        assert_eq!(srh.tag(), 0);
        assert_eq!(srh.protected(), false);
        assert_eq!(srh.segments().unwrap()[0], seg0);
        assert_eq!(srh.segments().unwrap()[1], seg1);

        // Test we can use iter (32 elements MAX implemented)
        let mut iter = srh.segments().unwrap().iter();
        assert_eq!(iter.next().unwrap(), &seg0);
        assert_eq!(iter.next().unwrap(), &seg1);

        // Insert header onto packet
        if let Ok(()) = v6pkt2.insert_header(NextHeader::Routing, &srh) {
            let srhpkt = v6pkt2.parse_header::<SRH<Ipv6Header>>();
            assert_eq!(srhpkt.get_header().segments().unwrap().len(), 2);
        } else {
            panic!("Error adding srh header onto v6 packet");
        }
    }

    let old_payload_len = v6pkt3.get_header().payload_len();
    if let Ok(()) = v6pkt3.insert_header(NextHeader::Routing, &srh) {
        println!("OK! Insert of SRH");
    } else {
        panic!("Error inserting test SRH");
    }

    {
        assert_eq!(
            v6pkt3.get_header().next_header().unwrap(),
            NextHeader::Routing
        );

        // manually add calculated srh offset and increase
        assert_eq!(v6pkt3.get_header().payload_len(), old_payload_len + 40);
    }

    {
        let mut srhv6_1 = v6pkt3.clone().parse_header::<SRH<Ipv6Header>>();
        let seg2 = Ipv6Addr::from_str("2001:db8:85a3::8a2e:0370:7337").unwrap();
        let seg3 = Ipv6Addr::from_str("2001:db8:85a3::8a2e:0370:7338").unwrap();
        let segs2 = vec![seg2, seg3];
        let srh2 = <SegmentRoutingHeader<Ipv6Header, U2>>::new(*GenericArray::<_, U2>::from_slice(
            &segs2[..],
        ));
        {
            let check_fn = |hdr: &mut SRH<Ipv6Header>| assert_eq!(hdr.segments().unwrap().len(), 2);
            if let Ok(diff) =
                srhv6_1.swap_header_fn::<SegmentRoutingHeader<Ipv6Header, U2>>(&srh2, &check_fn)
            {
                assert_eq!(diff, 0);
                let srh = srhv6_1.get_header();
                assert_eq!(srh.segments().unwrap()[0], seg2);
                assert_eq!(srh.segments().unwrap()[1], seg3);
            }
        }
    }

    {
        let mut srhv6_2 = v6pkt3.clone().parse_header::<SRH<Ipv6Header>>();
        let seg4 = Ipv6Addr::from_str("2001:db8:85a3::8a2e:0370:1000").unwrap();
        let segs3 = vec![seg4];
        let srh3 = <SegmentRoutingHeader<Ipv6Header, U1>>::new(*GenericArray::<_, U1>::from_slice(
            &segs3[..],
        ));
        {
            if let Ok(diff) = srhv6_2.swap_header::<SegmentRoutingHeader<Ipv6Header, U1>>(&srh3) {
                assert_eq!(diff, -16);
                let srh = srhv6_2.get_header();
                assert_eq!(srh.segments().unwrap().len(), 1);
                assert_eq!(srh.segments().unwrap()[0], seg4);
            }
        }
    }

    {
        let mut srhv6_3 = v6pkt3.clone().parse_header::<SRH<Ipv6Header>>();
        let seg5 = Ipv6Addr::from_str("2001:db8:85a3::8a2e:0370:3000").unwrap();
        let seg6 = Ipv6Addr::from_str("2001:db8:85a3::8a2e:0370:3001").unwrap();
        let seg7 = Ipv6Addr::from_str("2001:db8:85a3::8a2e:0370:3002").unwrap();
        let segs4 = vec![seg5, seg6, seg7];
        let srh4 = <SegmentRoutingHeader<Ipv6Header, U3>>::new(*GenericArray::<_, U3>::from_slice(
            &segs4[..],
        ));
        {
            if let Ok(diff) = srhv6_3.swap_header::<SegmentRoutingHeader<Ipv6Header, U3>>(&srh4) {
                assert_eq!(diff, 32);
                let srh = srhv6_3.get_header();
                assert_eq!(srh.segments().unwrap().len(), 3);
                assert_eq!(srh.segments().unwrap()[0], seg5);
                assert_eq!(srh.segments().unwrap()[1], seg6);
                assert_eq!(srh.segments().unwrap()[2], seg7);
            }
        }
    }
}

fn remove_srh() {
    let pkt = packet_from_bytes(&SRH_BYTES);

    // Check Ethernet header
    let epkt = pkt.parse_header::<MacHeader>();
    let mut v6pkt = epkt.parse_header::<Ipv6Header>();

    if let Ok(_diff) = v6pkt.remove_header::<SRH<Ipv6Header>>() {
        let tcp_pkt = v6pkt.parse_header::<TcpHeader<Ipv6Header>>();
        {
            let payload = tcp_pkt.get_payload();
            let tcp_hdr = tcp_pkt.get_header();
            assert_eq!(tcp_hdr.src_port(), 3464);
            assert_eq!(tcp_hdr.dst_port(), 1024);
            assert_eq!(tcp_hdr.seq_num(), 0);
            assert_eq!(tcp_hdr.ack_num(), 0);
            assert_eq!(tcp_hdr.window_size(), 10);
            assert_eq!(payload.len(), 40);
            assert_eq!(payload[39], 7);
        }
    }
}

fn remove_srh_with_fn() {
    let pkt = packet_from_bytes(&SRH_BYTES);

    // Check Ethernet header
    let epkt = pkt.parse_header::<MacHeader>();
    let mut v6pkt = epkt.parse_header::<Ipv6Header>();
    let old_payload_len = v6pkt.get_header().payload_len();

    if let Ok(()) = v6pkt.remove_header::<SRH<Ipv6Header>>() {
        assert_eq!(v6pkt.get_header().next_header().unwrap(), NextHeader::Tcp);
        assert_eq!(
            v6pkt.get_header().payload_len(),
            // calculate diff from setup SRH_BYTES
            (old_payload_len as isize - (6 * 8 + 8)) as u16
        );

        {
            let src = Ipv6Addr::from_str("2001:db8:85a3::1").unwrap();
            let dst = Ipv6Addr::from_str("2001:db8:85a3::8a2e:0370:7334").unwrap();
            let flow = v6pkt.get_header().flow().unwrap();
            assert_eq!(flow.src_ip, src);
            assert_eq!(flow.dst_ip, dst);
            assert_eq!(flow.src_port, 3464);
            assert_eq!(flow.dst_port, 1024);
            assert_eq!(flow.proto, TCP_NXT_HDR);
        }

        {
            let tcp_pkt = v6pkt.parse_header::<TcpHeader<Ipv6Header>>();
            let payload = tcp_pkt.get_payload();
            let tcp_hdr = tcp_pkt.get_header();
            assert_eq!(tcp_hdr.src_port(), 3464);
            assert_eq!(tcp_hdr.dst_port(), 1024);
            assert_eq!(tcp_hdr.seq_num(), 0);
            assert_eq!(tcp_hdr.ack_num(), 0);
            assert_eq!(tcp_hdr.window_size(), 10);
            assert_eq!(payload.len(), 40);
            assert_eq!(payload[39], 7);
        }
    }
}

#[test]
fn packet_tests() {
    setup();
    srh_from_bytes();
    v6_from_bytes();
    insert_static_srh_from_bytes();
    remove_srh();
    remove_srh_with_fn();
}
