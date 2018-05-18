extern crate generic_array;
extern crate netbricks;
use generic_array::typenum::U2;
use generic_array::GenericArray;
use netbricks::common::EmptyMetadata;
use netbricks::headers::*;
use netbricks::interface::{dpdk, new_packet, Packet};
use std::convert::From;
use std::net::Ipv6Addr;
use std::str::FromStr;
use std::sync::{Once, ONCE_INIT};

static EAL_INIT: Once = ONCE_INIT;

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
    let packet_header = [
        // --- Ethernet header ---
        // Destination MAC
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x01,
        // Source MAC
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x02,
        // EtherType (IPv6)
        0x86,
        0xDD,
        // --- IPv6 Header ---
        // Version, Traffic Class, Flow Label
        0x60,
        0x00,
        0x00,
        0x00,
        // Payload Length
        0x00,
        0x18,
        // Next Header (Routing = 43)
        0x2b,
        // Hop Limit
        0x02,
        // Source Address
        0x20,
        0x01,
        0x0d,
        0xb8,
        0x85,
        0xa3,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x01,
        // Dest Address
        0x20,
        0x01,
        0x0d,
        0xb8,
        0x85,
        0xa3,
        0x00,
        0x00,
        0x00,
        0x00,
        0x8a,
        0x2e,
        0x03,
        0x70,
        0x73,
        0x34,
        // --- SRv6 Header --
        // Next Header (UDP)
        0x11,
        // Hdr Ext Len (two segments, units of 8 octets or 64 bits)
        0x04,
        // Routing type (SRv6)
        0x04,
        // Segments left
        0x00,
        // Last entry
        0x01,
        // Flags
        0x00,
        // Tag
        0x00,
        0x00,
        // Segments: [0] 2001:0db8:85a3:0000:0000:8a2e:0370:7334
        0x20,
        0x01,
        0x0d,
        0xb8,
        0x85,
        0xa3,
        0x00,
        0x00,
        0x00,
        0x00,
        0x8a,
        0x2e,
        0x03,
        0x70,
        0x73,
        0x34,
        // Segments: [1] 2001:0db8:85a3:0000:0000:8a2e:0370:7335
        0x20,
        0x01,
        0x0d,
        0xb8,
        0x85,
        0xa3,
        0x00,
        0x00,
        0x00,
        0x00,
        0x8a,
        0x2e,
        0x03,
        0x70,
        0x73,
        0x35,
    ];
    let pkt = packet_from_bytes(&packet_header);

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
        assert_eq!(v6.payload_len(), 24);
        assert_eq!(v6.next_header().unwrap(), NextHeader::Routing);
        assert_eq!(v6.hop_limit(), 2);
        assert_eq!(Ipv6Addr::from(v6.src()), src);
        assert_eq!(Ipv6Addr::from(v6.dst()), dst);
    }

    // Check SRH
    let srhpkt = v6pkt.parse_header::<SRH<Ipv6Header>>();
    {
        let srh = srhpkt.get_header();
        let seg0 = Ipv6Addr::from_str("2001:db8:85a3::8a2e:0370:7334").unwrap();
        let seg1 = Ipv6Addr::from_str("2001:db8:85a3::8a2e:0370:7335").unwrap();
        assert_eq!(srh.next_header().unwrap(), NextHeader::Udp);
        assert_eq!(srh.hdr_ext_len(), 4);
        assert_eq!(srh.routing_type(), 4);
        assert_eq!(srh.segments_left(), 0);
        assert_eq!(srh.last_entry(), 1);
        assert_eq!(srh.protected(), false);
        assert_eq!(srh.oam(), false);
        assert_eq!(srh.alert(), false);
        assert_eq!(srh.hmac(), false);
        assert_eq!(srh.tag(), 0);
        assert_eq!(srh.segments().len(), 2);
        assert_eq!(srh.segments()[0], seg0);
        assert_eq!(srh.segments()[1], seg1);
    }
}

fn insert_static_srh_from_bytes() {
    let packet_header = [
        // --- Ethernet header ---
        // Destination MAC
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x01,
        // Source MAC
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x02,
        // EtherType (IPv6)
        0x86,
        0xDD,
        // --- IPv6 Header ---
        // Version, Traffic Class, Flow Label
        0x60,
        0x00,
        0x00,
        0x00,
        // Payload Length
        0x00,
        0x18,
        // Next Header (UDP = 17)
        0x11,
        // Hop Limit
        0x02,
        // Source Address
        0x20,
        0x01,
        0x0d,
        0xb8,
        0x85,
        0xa3,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x01,
        // Dest Address
        0x20,
        0x01,
        0x0d,
        0xb8,
        0x85,
        0xa3,
        0x00,
        0x00,
        0x00,
        0x00,
        0x8a,
        0x2e,
        0x03,
        0x70,
        0x73,
        0x34,
    ];
    let pkt = packet_from_bytes(&packet_header);
    let epkt = pkt.parse_header::<MacHeader>();
    let mut v6pkt = epkt.parse_header::<Ipv6Header>();
    let mut v6pkt2 = v6pkt.clone();
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
        assert_eq!(srh.segments().len(), 2);
        assert_eq!(srh.tag(), 0);
        assert_eq!(srh.protected(), false);
        assert_eq!(srh.segments()[0], seg0);
        assert_eq!(srh.segments()[1], seg1);

        // Test we can use iter (32 elements MAX implemented)
        let mut iter = srh.segments().iter();
        assert_eq!(iter.next().unwrap(), &seg0);
        assert_eq!(iter.next().unwrap(), &seg1);

        // Insert header onto packet
        if let Ok(()) = v6pkt2.insert_header(&srh) {
            let srhpkt = v6pkt2.parse_header::<SRH<Ipv6Header>>();
            assert_eq!(srhpkt.get_header().segments().len(), 2);
        } else {
            panic!("Error adding srh header onto v6 packet");
        }
    }
}

#[test]
fn packet_tests() {
    setup();
    srh_from_bytes();
    insert_static_srh_from_bytes();
}
