extern crate e2d2;
extern crate time;
extern crate simd;
use e2d2::io;
use e2d2::io::Act;
use e2d2::headers::*;
use std::net::*;
use std::convert::From;
//use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};
//use std::time::Duration;

const DST_MAC : [u8; 6] = [0x00, 0x0c, 0x29, 0x50, 0xa9, 0xfc];
const SRC_MAC : [u8; 6] = [0x00, 0x26, 0x16, 0x00, 0x00, 0xd2];
fn prepare_mac_header() -> MacHeader {
    let mut hdr = MacHeader::new();
    hdr.etype = u16::to_be(0x800);
    hdr.src = SRC_MAC;
    hdr.dst = DST_MAC;
    hdr
}

const DST_MAC2 : [u8; 6] = [0x00, 0x0c, 0x29, 0x50, 0xa9, 0x1];
const SRC_MAC2 : [u8; 6] = [0x00, 0x26, 0x16, 0x00, 0x00, 0x2];
fn prepare_mac_header2() -> MacHeader {
    let mut hdr = MacHeader::new();
    hdr.etype = u16::to_be(0x800);
    hdr.src = SRC_MAC2;
    hdr.dst = DST_MAC2;
    hdr
}

fn prepare_ip_header(src:u8, dst:u8) -> IpHeader {
    let mut iphdr = IpHeader::new();
    iphdr.set_ttl(64);
    iphdr.set_ihl(5);
    iphdr.set_length(28);
    iphdr.set_protocol(0x11);
    iphdr.set_version(4);
    iphdr.set_src(u32::from(Ipv4Addr::new(192, 168, 0, src)));
    iphdr.set_dst(u32::from(Ipv4Addr::new(192, 168, 0, dst)));
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

fn prepare_udp_header2() -> UdpHeader {
    let mut udp_hdr = UdpHeader::new();
    udp_hdr.set_src_port(22);
    udp_hdr.set_dst_port(50);
    udp_hdr.set_length(8);
    udp_hdr.set_checksum(0xa722);
    udp_hdr
}

const CONVERSION_FACTOR:u64 = 1000000000;
fn send_thread(port: io::PmdPort, queue: i32, core: i32) {
    io::init_thread(core, core);
    println!("Sending started");
    let mut batch = io::PacketBatch::new(4);
    let mut tx = 0;
    let mut drops = 0;
    let mut cycles = 0;
    //let iphdr = prepare_ip_header(22, 233);
    //let iphdr2 = prepare_ip_header(12, 2);
    //let udphdr = prepare_udp_header();
    //let udphdr2 = prepare_udp_header2();
    let machdr = prepare_mac_header();
    //let machdr2 = prepare_mac_header2();
    let mut start = time::precise_time_ns() / CONVERSION_FACTOR;
    loop {
        let _ = batch.allocate_batch_with_size(60);

        batch.parse::<MacHeader>().replace(&machdr).act();
            //.parse::<IpHeader>()
            //.replace(&iphdr)
            //.parse::<UdpHeader>()
            //.replace(&udphdr).act();

        let sent = match port.send_queue(queue, &mut batch) {
            Ok(v) => v as usize,
            Err(e) => {
                println!("Error {:?}", e);
                0}
        };
        tx += sent;
        drops += batch.available();
        cycles += 1;
        let _ = batch.deallocate_batch();

        let now = time::precise_time_ns() / CONVERSION_FACTOR;
        if now > start {
            println!("{} tx_core {} pps {} drops {} loops {}", 
                     (now - start), core, tx, drops, cycles);
            tx = 0;
            cycles = 0;
            drops = 0;
            start = now;
        }
        let _ = batch.deallocate_batch();
    }
}

fn recv_thread(port: io::PmdPort, queue: i32, core: i32) {
    io::init_thread(core, core);
    println!("Receiving started");
    let mut batch = io::PacketBatch::new(32);
    let mut cycles = 0;
    let mut rx = 0;
    let mut no_rx = 0;
    let mut start = time::precise_time_ns() / CONVERSION_FACTOR;
    loop {
        let recv = match port.recv_queue(queue, &mut batch) {
            Ok(v) => v as usize,
            Err(_) => 0
        };
        cycles += 1;
        rx += recv;
        if recv == 0 {
            no_rx += 1
        }
        let _ = batch.deallocate_batch();
        let now = time::precise_time_ns() / CONVERSION_FACTOR;
        if now > start {
            println!("{} rx_core {} pps {} no_rx {} loops {}", 
                     (now - start), core, rx, no_rx, cycles);
            rx = 0;
            no_rx = 0;
            cycles = 0;
            start = now;
        }
        let _ = batch.deallocate_batch();
    }
}

fn main() {
    io::init_system_wl(11, &vec![String::from("82:00.0"),
                                String::from("82:00.1")]);
    let send_port0 =  io::PmdPort::new_mq_port(0, 1, 1, &vec![12], &vec![12]).unwrap();
    let send_port1 =  io::PmdPort::new_mq_port(1, 1, 1, &vec![13], &vec![13]).unwrap();
    let s0 = std::thread::spawn(move || {send_thread(send_port0, 0, 12)});
    let _ = std::thread::spawn(move || {send_thread(send_port1, 0, 13)});
    let _ = s0.join();
}
