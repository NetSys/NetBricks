extern crate e2d2;
extern crate time;
extern crate simd;
use e2d2::io;
use e2d2::io::Act;
use e2d2::headers::*;
use std::net::*;
use std::convert::From;

const DST_MAC : [u8; 6] = [0x00, 0x0c, 0x29, 0x50, 0xa9, 0xfc];
const SRC_MAC : [u8; 6] = [0x00, 0x26, 0x16, 0x00, 0x00, 0xd2];
fn prepare_mac_header() -> MacHeader {
    let mut hdr = MacHeader::new();
    hdr.etype = u16::to_be(0x800);
    hdr.src = SRC_MAC;
    hdr.dst = DST_MAC;
    hdr
}

fn prepare_ip_header() -> IpHeader {
    let mut iphdr = IpHeader::new();
    iphdr.set_ttl(64);
    iphdr.set_ihl(5);
    iphdr.set_length(28);
    iphdr.set_protocol(0x11);
    iphdr.set_version(4);
    iphdr.set_src(u32::from(Ipv4Addr::new(192, 168, 0, 101)));
    iphdr.set_dst(u32::from(Ipv4Addr::new(192, 168, 0, 10)));
    iphdr.set_flags(0);
    iphdr.set_id(0);
    iphdr.set_fragment_offset(0);
    iphdr.set_csum(0xf900);
    iphdr
}

fn prepare_udp_header() -> UdpHeader {
    let mut udp_hdr = UdpHeader::new();
    udp_hdr.set_src_port(49905);
    udp_hdr.set_dst_port(5096);
    udp_hdr.set_length(8);
    udp_hdr.set_checksum(0xa722);
    udp_hdr
}

fn main() {
    io::init_system(1);
    //IpHeader::show_offsets();
    let mut batch = io::PacketBatch::new(32);
    let (send_port_ret, recv_port_ret) = io::PmdPort::new_loopback_port(0, 1);
    let send_port = send_port_ret.unwrap();
    let recv_port = recv_port_ret.unwrap();
    //let send_port = io::PmdPort::new_simple_port(0, 1).unwrap();

    //let recv_port = 
        //if cfg!(feature = "recv") {
             //io::PmdPort::new_simple_port(1, 1).unwrap()
        //} else {
            //io::PmdPort::null_port().unwrap()
        //};
    let conversion_factor:u64 = 1000000000;
    let mut start = time::precise_time_ns() / conversion_factor;
    let mut rx:u64 = 0;
    let mut tx:u64 = 0;
    let iphdr = prepare_ip_header();
    let udphdr = prepare_udp_header();
    let machdr = prepare_mac_header();
    println!("Header {}", iphdr);
    println!("Header {}", udphdr);
    loop {
        let _ = batch.allocate_batch_with_size(60).unwrap();

        batch.parse::<MacHeader>().replace(&machdr)
            .parse::<IpHeader>()
            .replace(&iphdr)
            .parse::<UdpHeader>()
            .replace(&udphdr).act();

        if cfg!(feature = "send") {
            let sent = send_port.send(&mut batch).unwrap();
            tx += sent as u64;
        } else {
            tx += batch.available() as u64;
        }
        let _ = batch.deallocate_batch().unwrap();

        if cfg!(feature = "recv") {
            let recv = recv_port.recv(&mut batch).unwrap();
            rx += recv as u64;
            if cfg!(feature = "print") {
                batch.dump::<MacHeader>();
            }
            let _ = batch.deallocate_batch().unwrap();
        }

        let now = time::precise_time_ns() / conversion_factor;
        if now != start {
            print!("{} ", (now - start));
            print!("tx {} ", tx);
            print!("rx {} ", rx);
            println!("");
            start = time::precise_time_ns() / conversion_factor;
            tx = 0;
            rx = 0;
        }
    }
}
