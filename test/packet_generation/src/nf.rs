use std::net::Ipv4Addr;
use std::str::FromStr;
use e2d2::headers::*;
use e2d2::interface::*;
use e2d2::queues::*;
use e2d2::scheduler::*;

pub struct PacketCreator {
    mac: MacHeader,
    ip: IpHeader,
    producer: MpscProducer,
}

impl PacketCreator {
    pub fn new(producer: MpscProducer) -> PacketCreator {
        let mut mac = MacHeader::new();
        mac.dst = [0x68, 0x05, 0xca, 0x00, 0x00, 0xac];
        mac.src = [0x68, 0x05, 0xca, 0x00, 0x00, 0x01];
        mac.set_etype(0x0800);
        let mut ip = IpHeader::new();
        ip.set_src(u32::from(Ipv4Addr::from_str("10.0.0.1").unwrap()));
        ip.set_dst(u32::from(Ipv4Addr::from_str("10.0.0.5").unwrap()));
        ip.set_ttl(128);
        ip.set_version(4);
        ip.set_ihl(5);
        ip.set_length(20);
        PacketCreator { mac : mac, ip : ip, producer: producer, }
    }

    #[inline]
    pub fn create_packet(&self) -> Packet<IpHeader> {
        new_packet().unwrap().push_header(&self.mac).unwrap().push_header(&self.ip).unwrap()
    }
}

impl Executable for PacketCreator {
    fn execute(&mut self) {
        for _ in 0..16 {
            self.producer.enqueue_one(&mut self.create_packet());
        }
        //let mut vec = Vec::with_capacity(16);
        //vec.extend((0..16).map(|_| self.create_packet()));
        //{
            //self.producer.enqueue(&mut vec[..]);
        //}
    }
}
