use netbricks::common::*;
use netbricks::headers::*;
use netbricks::interface::*;
use netbricks::queues::*;
use netbricks::scheduler::*;
use std::net::Ipv4Addr;
use std::str::FromStr;

pub struct PacketCreator {
    mac: MacHeader,
    ip: Ipv4Header,
    producer: MpscProducer,
}

impl PacketCreator {
    pub fn new(producer: MpscProducer) -> PacketCreator {
        let mut mac = MacHeader::new();
        let dst = MacAddress {
            addr: [0x68, 0x05, 0xca, 0x00, 0x00, 0xac],
        };
        let src = MacAddress {
            addr: [0x68, 0x05, 0xca, 0x00, 0x00, 0x01],
        };

        mac.set_dst(dst);
        mac.set_src(src);
        mac.set_etype(mac::EtherType::IPv4);

        let mut ip = Ipv4Header::new();
        ip.set_src(Ipv4Addr::from_str("10.0.0.1").unwrap());
        ip.set_dst(Ipv4Addr::from_str("10.0.0.5").unwrap());
        ip.set_ttl(128);
        ip.set_version(4);
        ip.set_ihl(5);
        ip.set_length(20);

        PacketCreator {
            mac: mac,
            ip: ip,
            producer: producer,
        }
    }

    #[inline]
    fn initialize_packet(
        &self,
        pkt: Packet<NullHeader, EmptyMetadata>,
    ) -> Packet<Ipv4Header, EmptyMetadata> {
        pkt.push_header(&self.mac)
            .unwrap()
            .push_header(&self.ip)
            .unwrap()
    }

    #[inline]
    pub fn create_packet(&self) -> Packet<Ipv4Header, EmptyMetadata> {
        self.initialize_packet(new_packet().unwrap())
    }
}

impl Executable for PacketCreator {
    fn execute(&mut self) {
        for _ in 0..16 {
            self.producer.enqueue_one(self.create_packet());
        }
    }
    fn dependencies(&mut self) -> Vec<usize> {
        vec![]
    }
}
