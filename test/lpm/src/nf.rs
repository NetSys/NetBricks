use e2d2::common::EmptyMetadata;
use e2d2::headers::*;
use e2d2::operators::*;
use e2d2::scheduler::*;
use fnv::FnvHasher;
use std::collections::HashMap;
use std::convert::From;
use std::hash::BuildHasherDefault;
use std::net::Ipv4Addr;

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
                    let base = ((t24entry & (!OVERFLOW_MASK)) as usize) << 8;
                    let start = base + (addr & 0xff);
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

pub fn lpm<T: 'static + Batch<Header = NullHeader, Metadata = EmptyMetadata>, S: Scheduler + Sized>
    (parent: T,
     s: &mut S)
     -> CompositionBatch {
    let mut lpm_table = IPLookup::new();
    lpm_table.insert_ipv4(&Ipv4Addr::new(188, 19, 50, 135), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(123, 19, 205, 58), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(58, 218, 199, 165), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(61, 90, 38, 155), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(190, 179, 91, 29), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(198, 23, 250, 66), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(42, 103, 111, 67), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(117, 197, 187, 144), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(207, 198, 106, 183), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(122, 90, 22, 43), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(5, 167, 64, 38), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(152, 166, 114, 31), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(191, 81, 59, 58), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(190, 175, 182, 182), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(190, 237, 89, 70), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(181, 21, 43, 134), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(122, 171, 197, 247), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(113, 212, 69, 195), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(151, 74, 149, 41), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(27, 28, 145, 139), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(206, 169, 145, 35), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(105, 103, 122, 43), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(116, 45, 32, 133), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(202, 109, 166, 177), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(195, 53, 118, 92), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(5, 167, 65, 10), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(216, 151, 137, 247), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(103, 56, 182, 207), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(216, 151, 138, 92), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(88, 250, 194, 52), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(41, 232, 211, 169), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(106, 215, 170, 250), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(187, 3, 143, 162), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(110, 85, 81, 79), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(220, 111, 212, 68), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(185, 151, 210, 156), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(190, 179, 180, 225), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(2, 92, 39, 245), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(107, 191, 202, 53), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(46, 161, 9, 22), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(122, 117, 64, 197), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(186, 133, 153, 160), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(36, 255, 211, 72), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(173, 234, 225, 96), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(95, 68, 215, 116), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(115, 84, 82, 217), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(84, 164, 153, 138), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(190, 50, 251, 94), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(93, 105, 249, 221), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(37, 21, 101, 156), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(143, 0, 222, 235), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(5, 167, 69, 45), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(37, 191, 159, 133), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(203, 134, 213, 65), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(84, 211, 75, 56), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(117, 248, 162, 119), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(69, 178, 195, 20), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(113, 189, 147, 99), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(185, 84, 202, 52), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(188, 143, 232, 190), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(41, 221, 50, 102), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(92, 82, 174, 27), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(113, 55, 12, 76), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(62, 211, 191, 202), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(191, 82, 178, 107), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(181, 23, 198, 10), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(117, 196, 200, 35), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(117, 198, 60, 179), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(178, 136, 73, 133), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(203, 130, 228, 60), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(179, 41, 211, 34), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(193, 92, 162, 10), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(216, 151, 130, 236), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(2, 185, 182, 1), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(185, 148, 100, 45), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(85, 105, 157, 175), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(171, 80, 158, 237), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(109, 165, 67, 16), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(5, 167, 66, 53), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(183, 165, 159, 231), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(152, 232, 193, 27), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(60, 184, 112, 190), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(122, 163, 104, 12), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(177, 213, 230, 190), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(113, 146, 90, 33), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(190, 49, 119, 180), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(89, 187, 144, 19), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(31, 168, 81, 84), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(201, 254, 169, 36), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(200, 71, 53, 112), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(90, 189, 133, 179), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(24, 219, 68, 17), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(190, 178, 143, 49), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(41, 105, 236, 253), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(183, 60, 48, 25), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(188, 143, 232, 254), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(36, 97, 169, 81), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(192, 99, 147, 251), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(171, 233, 174, 141), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(112, 227, 158, 252), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(188, 143, 233, 59), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(192, 209, 125, 92), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(5, 167, 66, 37), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(95, 215, 103, 88), 32, 1);
    lpm_table.insert_ipv4(&Ipv4Addr::new(5, 167, 65, 50), 32, 1);
    lpm_table.construct_table();
    let mut groups = parent.parse::<MacHeader>()
        .transform(box |p| p.get_mut_header().swap_addresses())
        .parse::<IpHeader>()
        .group_by(3,
                  box move |pkt| {
                      let hdr = pkt.get_header();
                      lpm_table.lookup_entry(hdr.src()) as usize
                  },
                  s);
    let pipeline =
        merge(vec![groups.get_group(0).unwrap(), groups.get_group(1).unwrap(), groups.get_group(2).unwrap()]).compose();
    pipeline
}
