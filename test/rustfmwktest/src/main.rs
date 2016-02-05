extern crate e2d2;
extern crate time;
extern crate simd;
use e2d2::io;
use e2d2::io::Act;
use e2d2::headers::*;
use std::net::*;
use std::convert::From;

const SRC_MAC : [u8; 6] = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06];
const DST_MAC : [u8; 6] = [0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c];
#[inline]
fn set_ether_type(hdr: &mut MacHeader) {
    let t:u16 = 0x0800;
    hdr.etype = t.to_be();
    hdr.src = SRC_MAC;
    hdr.dst = DST_MAC;
}

fn prepare_header() -> IpHeader {
    let mut iphdr = IpHeader::new();
    iphdr.set_ttl(64);
    iphdr.set_version(4);
    iphdr.set_ihl(5);
    iphdr.set_length(40);
    iphdr.set_protocol(0x11);
    iphdr.set_src(u32::from(Ipv4Addr::new(10, 0, 0, 2)));
    iphdr.set_dst(u32::from(Ipv4Addr::new(10, 1, 0, 1)));
    iphdr.set_flags(0x2);
    iphdr.set_id(32);
    iphdr.set_fragment_offset(2);
    iphdr
}

fn main() {
    io::init_system(1);
    //IpHeader::show_offsets();
    let mut batch = io::PacketBatch::new(32);
    let send_port = io::PmdPort::new_simple_port(0, 1).unwrap();

    let recv_port = 
        if cfg!(feature = "recv") {
             io::PmdPort::new_simple_port(1, 1).unwrap()
        } else {
            io::PmdPort::null_port().unwrap()
        };
    let conversion_factor:u64 = 1000000000;
    let mut start = time::precise_time_ns() / conversion_factor;
    let mut rx:u64 = 0;
    let mut tx:u64 = 0;
    let iphdr = prepare_header();
    println!("Header {}", iphdr);
    loop {
        let _ = batch.allocate_batch_with_size(60).unwrap();

        batch.parse::<MacHeader>().
            transform(&set_ether_type).parse::<IpHeader>()
            .transform(&|hdr| hdr.apply(&iphdr)).act();

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
