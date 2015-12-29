extern crate e2d2;
extern crate time;
use e2d2::io;
use e2d2::headers;
const SRC_MAC : [u8; 6] = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06];
const DST_MAC : [u8; 6] = [0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c];
#[inline]
fn set_ether_type(hdr: &mut headers::MacHeader) {
    let t:u16 = 0x0800;
    hdr.etype = t.to_be();
    hdr.src = SRC_MAC;
    hdr.dst = DST_MAC;
}

#[inline]
fn set_ip_header(hdr: &mut headers::IpHeader) {
    hdr.set_version(4);
    hdr.set_header_len(5);
    hdr.len = u16::to_be(20);
    hdr.ttl = 64;
    hdr.protocol = 0x11;
}

fn main() {
    io::init_system(1);
    let mut batch = io::PacketBatch::new(32);
    let mut offsets = Vec::<usize>::with_capacity(32);
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
    loop {
        if cfg!(feature = "send") {
            let _ = batch.allocate_batch_with_size(60).unwrap();
            batch.transform(&set_ether_type);
            batch.offsets_efficient::<headers::MacHeader>(&mut offsets);
            batch.transform_at_offset(&offsets, &set_ip_header);
            offsets.clear();
            let sent = send_port.send(&mut batch).unwrap();
            tx += sent as u64;
            let _ = batch.deallocate_batch().unwrap();
        }
        if cfg!(feature = "recv") {
            let recv = recv_port.recv(&mut batch).unwrap();
            rx += recv as u64;
            batch.offsets_efficient::<headers::MacHeader>(&mut offsets);
            if cfg!(feature = "print") {
                batch.dump::<headers::MacHeader>();
                batch.dump_at_offset::<headers::IpHeader>(&offsets);
            }
            let _ = batch.deallocate_batch().unwrap();
        }
        let now = time::precise_time_ns() / conversion_factor;
        if now != start {
            print!("{} ", (now - start));
            if cfg!(feature = "send") {
                print!("tx {} ", tx);
            }
            if cfg!(feature = "recv") {
                print!("rx {} ", rx);
            }
            println!("");
            start = time::precise_time_ns() / conversion_factor;
            tx = 0;
            rx = 0;
        }
    }
}
