extern crate generic_array;
#[macro_use]
extern crate netbricks;
use generic_array::typenum::*;
use generic_array::GenericArray;
use netbricks::common::EmptyMetadata;
use netbricks::headers::*;
use netbricks::interface::{new_packet, Packet};
use netbricks::tests::*;
use netbricks::utils::*;
use std::convert::From;
use std::net::Ipv6Addr;
use std::str::FromStr;

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

#[test]
fn ndp_router_advertisement_from_bytes() {
    dpdk_test! {
        let pkt = packet_from_bytes(&ICMP_ROUTER_ADVERTISEMENT_BYTES);
        // Check Ethernet header
        let epkt = pkt.parse_header::<MacHeader>();
        {
            let eth = epkt.get_header();
            assert_eq!(eth.dst().addr, MacAddress::new(0, 0, 0, 0, 0, 1).addr);
            assert_eq!(eth.src().addr, MacAddress::new(0, 0, 0, 0, 0, 2).addr);
            assert_eq!(eth.etype(), Some(EtherType::IPv6));
        }

         // Check IPv6 header
        let v6pkt = epkt.parse_header::<Ipv6Header>();
        let &v6 = v6pkt.get_header();
        let payload_len = v6.payload_len();
        {
            let v6 = v6pkt.get_header();
            let src = Ipv6Addr::from_str("fe80::d4f0:45ff:fe0c:664b").unwrap();
            let dst = Ipv6Addr::from_str("ff02::1").unwrap();
            assert_eq!(v6.version(), 6);
            assert_eq!(v6.traffic_class(), 0);
            assert_eq!(v6.flow_label(), 0);
            assert_eq!(payload_len, 64);
            assert_eq!(v6.next_header().unwrap(), NextHeader::Icmp);
            assert_eq!(v6.hop_limit(), 255);
            assert_eq!(Ipv6Addr::from(v6.src()), src);
            assert_eq!(Ipv6Addr::from(v6.dst()), dst);
        }

        //Check Icmp header
        let icmp_pkt = v6pkt.parse_header::<Icmpv6RouterAdvertisement<Ipv6Header>>();
        {
            let icmpv6h = icmp_pkt.get_header();
            assert_eq!(icmpv6h.msg_type().unwrap(), IcmpMessageType::RouterAdvertisement);
            assert_eq!(icmpv6h.checksum(), 0xf50c);
            assert_eq!(icmpv6h.code(), 0);
            assert_eq!(icmpv6h.current_hop_limit(), 64);
            assert_eq!(icmpv6h.managed_addr_cfg(), false);
            assert_eq!(icmpv6h.other_cfg(), true);
            assert_eq!(icmpv6h.router_lifetime(), 1800);
            assert_eq!(icmpv6h.reachable_time(), 2055);
            assert_eq!(icmpv6h.retrans_timer(), 1500);
         //   let expected_mac_address = MacAddress::from_str("c2:00:54:f5:00:00").unwrap();
          //  let options = icmpv6h.parse_options(payload_len);
           // let source_link_layer_address = icmpv6h.source_link_layer_address(options);
           // assert_eq!(source_link_layer_address.unwrap(), expected_mac_address);
        }
    }
}

#[test]
fn ndp_router_advertisement_from_bytes_no_link_layer_address() {
    dpdk_test! {
        let pkt = packet_from_bytes(&ICMP_ROUTER_ADVERTISEMENT_BYTES_NO_LINK_LAYER_ADDRESS  );
        // Check Ethernet header
        let epkt = pkt.parse_header::<MacHeader>();
        {
            let eth = epkt.get_header();
            assert_eq!(eth.dst().addr, MacAddress::new(0, 0, 0, 0, 0, 1).addr);
            assert_eq!(eth.src().addr, MacAddress::new(0, 0, 0, 0, 0, 2).addr);
            assert_eq!(eth.etype(), Some(EtherType::IPv6));
        }

         // Check IPv6 header
        let v6pkt = epkt.parse_header::<Ipv6Header>();
        let &v6 = v6pkt.get_header();
        let payload_len = v6.payload_len();
        {
            let v6 = v6pkt.get_header();
            let src = Ipv6Addr::from_str("fe80::d4f0:45ff:fe0c:664b").unwrap();
            let dst = Ipv6Addr::from_str("ff02::1").unwrap();
            assert_eq!(v6.version(), 6);
            assert_eq!(v6.traffic_class(), 0);
            assert_eq!(v6.flow_label(), 0);
            assert_eq!(v6.payload_len(), 56);
            assert_eq!(v6.next_header().unwrap(), NextHeader::Icmp);
            assert_eq!(v6.hop_limit(), 255);
            assert_eq!(Ipv6Addr::from(v6.src()), src);
            assert_eq!(Ipv6Addr::from(v6.dst()), dst);
        }

        //Check Icmp header
        let icmp_pkt = v6pkt.parse_header::<Icmpv6RouterAdvertisement<Ipv6Header>>();
        {
            let icmpv6h = icmp_pkt.get_header();
            assert_eq!(icmpv6h.msg_type().unwrap(), IcmpMessageType::RouterAdvertisement);
            assert_eq!(icmpv6h.checksum(), 0xf50c);
            assert_eq!(icmpv6h.code(), 0);
            assert_eq!(icmpv6h.current_hop_limit(), 64);
            assert_eq!(icmpv6h.managed_addr_cfg(), false);
            assert_eq!(icmpv6h.other_cfg(), true);
            assert_eq!(icmpv6h.router_lifetime(), 1800);
            assert_eq!(icmpv6h.reachable_time(), 2055);
            assert_eq!(icmpv6h.retrans_timer(), 1500);
        //    let options = icmpv6h.parse_options(payload_len);
          //  let source_link_layer = options.get(&NDPOptionType::SourceLinkLayerAddress);
           // assert_eq!(source_link_layer.is_some(), false);
        }
    }
}

#[test]
fn icmpv6_too_big_from_bytes() {
    dpdk_test! {
        let pkt = packet_from_bytes(&ICMP_TOO_BIG_BYTES);
        let epkt = pkt.parse_header::<MacHeader>();
        {
            let eth = epkt.get_header();
            assert_eq!(eth.dst().addr, MacAddress::new(96, 3, 8, 162, 88, 156).addr);
            assert_eq!(eth.src().addr, MacAddress::new(124, 154, 84, 106, 238, 254).addr);
            assert_eq!(eth.etype(), Some(EtherType::IPv6));
        }

        let v6pkt = epkt.parse_header::<Ipv6Header>();
        let icmpv6_pkt = v6pkt.parse_header::<Icmpv6PktTooBig<Ipv6Header>>();
        let icmpv6h = icmpv6_pkt.get_header();
        {
            assert_eq!(icmpv6h.msg_type().unwrap(), IcmpMessageType::PacketTooBig);
            assert_eq!(icmpv6h.checksum(), 0x5652);
            assert_eq!(icmpv6h.code(), 0);
            assert_eq!(icmpv6h.mtu(), 1280);
            assert_eq!(icmpv6_pkt.get_payload().len(), icmpv6h.payload_size(0));
        }


        // See if inner-ipv6 hdr (offending header) is actually ipv6 hdr
        {
            let evilv6_payload = icmpv6_pkt.get_payload();
            // Convert &[u8] (get_payload) into associated type
            let evilv6h: Ipv6Header = cast_payload::<Ipv6Header>(evilv6_payload);
            let src = Ipv6Addr::from_str("2601:449:4200:359a:bd78:6aaf:4f22:b652").unwrap();
            let dst = Ipv6Addr::from_str("2001:470:1:18::1281").unwrap();
            assert_eq!(evilv6h.version(), 6);
            assert_eq!(evilv6h.traffic_class(), 0);
            assert_eq!(evilv6h.flow_label(), 382544);
            assert_eq!(evilv6h.payload_len(), 1408);
            assert_eq!(evilv6h.next_header().unwrap(), NextHeader::Icmp);
            assert_eq!(evilv6h.hop_limit(),55);
            assert_eq!(Ipv6Addr::from(evilv6h.src()), src);
            assert_eq!(Ipv6Addr::from(evilv6h.dst()), dst);
        }
    }
}

#[test]
fn srh_from_bytes() {
    dpdk_test! {
        let pkt = packet_from_bytes(&SRH_BYTES);

        // Check Ethernet header
        let epkt = pkt.parse_header::<MacHeader>();
        {
            let eth = epkt.get_header();
            assert_eq!(eth.dst().addr, MacAddress::new(0, 0, 0, 0, 0, 1).addr);
            assert_eq!(eth.src().addr, MacAddress::new(0, 0, 0, 0, 0, 2).addr);
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
}

#[test]
fn v6_from_bytes() {
    dpdk_test! {
        let pkt = packet_from_bytes(&V6_BYTES);

        // Check Ethernet header
        let epkt = pkt.parse_header::<MacHeader>();
        {
            let eth = epkt.get_header();
            assert_eq!(eth.dst().addr, MacAddress::new(0, 0, 0, 0, 0, 1).addr);
            assert_eq!(eth.src().addr, MacAddress::new(0, 0, 0, 0, 0, 2).addr);
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
}

#[test]
fn insert_static_srh_from_bytes() {
    dpdk_test! {
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
            if let Ok(()) = v6pkt2.insert_v6_header(NextHeader::Routing, &srh) {
                let srhpkt = v6pkt2.parse_header::<SRH<Ipv6Header>>();
                assert_eq!(srhpkt.get_header().segments().unwrap().len(), 2);
            } else {
                panic!("Error adding srh header onto v6 packet");
            }
        }

        let old_payload_len = v6pkt3.get_header().payload_len();
        if let Ok(()) = v6pkt3.insert_v6_header(NextHeader::Routing, &srh) {
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
}

#[test]
fn remove_srh() {
    dpdk_test! {
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
}

#[test]
fn remove_srh_with_fn() {
    dpdk_test! {
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
}
