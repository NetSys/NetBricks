use e2d2::headers::*;
use e2d2::operators::*;
use e2d2::scheduler::*;
use fnv::FnvHasher;
use std::net::Ipv4Addr;
use std::convert::From;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;

type FnvHash = BuildHasherDefault<FnvHasher>;
pub struct IPLookup {
    tbl24: Vec<u16>,
    tbl_long: Vec<u16>,
    current_tbl_long: usize,
    raw_entries: Vec<HashMap<u32, u16, FnvHash>>,
}

const TBL24_SIZE: usize = ((1 << 24) + 1);
const RAW_SIZE: usize = 33;
const OVERFLOW_MASK: u16 = 0x8000;
#[derive(Default, Clone)]
struct Empty;
impl Default for IPLookup {
    fn default() -> IPLookup {
        IPLookup {
            tbl24: (0..TBL24_SIZE).map(|_| 0).collect(),
            tbl_long: (0..TBL24_SIZE).map(|_| 0).collect(),
            current_tbl_long: 0,
            raw_entries: (0..RAW_SIZE).map(|_| Default::default()).collect(),
        }
    }
}

impl IPLookup {
    pub fn new() -> IPLookup {
        Default::default()
    }

    pub fn insert_ipv4(&mut self, ip: &Ipv4Addr, len: usize, gate: u16) {
        let ip_u32 = u32::from(*ip);
        self.insert(ip_u32, len, gate);
    }

    pub fn insert(&mut self, ip: u32, len: usize, gate: u16) {
        self.raw_entries[len].insert(ip, gate);
    }

    pub fn construct_table(&mut self) {
        for i in 0..25 {
            for (k, v) in &self.raw_entries[i] {
                let start = (k >> 8) as usize;
                let end = (start + (1 << (24 - i))) as usize;
                for pfx in start..end {
                    self.tbl24[pfx] = *v;
                }
            }
        }
        for i in 25..RAW_SIZE {
            for (k, v) in &self.raw_entries[i] {
                let addr = *k as usize;
                let t24entry = self.tbl24[addr >> 8];
                if (t24entry & OVERFLOW_MASK) == 0 {
                    // Not overflown and entered yet
                    let ctlb = self.current_tbl_long;
                    let start = ctlb + (addr & 0xff); // Look at last 8 bits (since first 24 are predetermined.
                    let end = start + (1 << (32 - i));
                    for j in ctlb..(ctlb + 256) {
                        if j < start || j >= end {
                            self.tbl_long[j] = t24entry;
                        } else {
                            self.tbl_long[j] = *v;
                        }
                    }
                    self.tbl24[addr >> 8] = ((ctlb >> 8) as u16) | OVERFLOW_MASK;
                    self.current_tbl_long += 256;
                } else {
                    let start = ((t24entry & (!OVERFLOW_MASK)) as usize) << 8 + (addr & 0xff);
                    let end = start + (1 << (32 - i));
                    for j in start..end {
                        self.tbl_long[j] = *v;
                    }
                }
            }
        }
    }

    #[inline]
    pub fn lookup_entry(&self, ip: u32) -> u16 {
        let addr = ip as usize;
        let t24entry = self.tbl24[addr >> 8];
        if (t24entry & OVERFLOW_MASK) > 0 {
            let index = ((t24entry & !OVERFLOW_MASK) as usize) << 8 + (addr & 0xff);
            self.tbl_long[index]
        } else {
            t24entry
        }
    }
}

pub fn lpm<T: 'static + Batch<Header = NullHeader>>(parent: T, s: &mut Scheduler) -> CompositionBatch<NullHeader> {
    let mut lpm_table = IPLookup::new();
    lpm_table.insert_ipv4(&Ipv4Addr::new(0, 0, 0, 0), 0, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(10, 0, 0, 0), 8, 2);
    lpm_table.insert_ipv4(&Ipv4Addr::new(192, 0, 0, 0), 8, 2);
    lpm_table.construct_table();
    let mut groups = parent.parse::<MacHeader>()
                           .parse::<IpHeader>()
                           .group_by(3,
                                    box move |pkt| {
                                        let hdr = pkt.get_header();
                                        lpm_table.lookup_entry(hdr.src()) as usize
                                    }, s);
    let pipeline = merge(vec![groups.get_group(0).unwrap(),
                              groups.get_group(1).unwrap(),
                              groups.get_group(2).unwrap()])
                       .compose();
    pipeline
}
