use byteorder::{BigEndian, ByteOrder};
use fnv::FnvHasher;
use native::zcsi::*;
use std::hash::Hasher;
use std::mem;
use std::slice;

// FIXME: Currently just deriving Hash, but figure out if this is a performance problem. By default, Rust uses SipHash
// which is supposed to have reasonable performance characteristics.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(C, packed)]
pub struct Flow {
    pub src_ip: u32,
    pub dst_ip: u32,
    pub src_port: u16,
    pub dst_port: u16,
    pub proto: u8,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Ipv4Prefix {
    pub ip_address: u32,
    pub prefix: u8,
    mask: u32, /* min_address: u32,
                * max_address: u32, */
}

impl Ipv4Prefix {
    pub fn new(address: u32, prefix: u8) -> Ipv4Prefix {
        let mask = if prefix == 0 {
            0
        } else {
            let inv_pfx = 32 - prefix;
            !((1u32 << (inv_pfx as u32)) - 1)
        };
        Ipv4Prefix {
            ip_address: address & mask,
            prefix: prefix,
            mask: mask,
        }
    }

    #[inline]
    pub fn in_range(&self, address: u32) -> bool {
        (address & self.mask) == self.ip_address
    }
}

const IHL_TO_BYTE_FACTOR: usize = 4; // IHL is in terms of number of 32-bit words.

/// This assumes the function is given the Mac Payload
#[inline]
pub fn ipv4_extract_flow(bytes: &[u8]) -> Option<Flow> {
    let port_start = (bytes[0] & 0xf) as usize * IHL_TO_BYTE_FACTOR;
    Some(Flow {
        proto: bytes[9],
        src_ip: BigEndian::read_u32(&bytes[12..16]),
        dst_ip: BigEndian::read_u32(&bytes[16..20]),
        src_port: BigEndian::read_u16(&bytes[(port_start)..(port_start + 2)]),
        dst_port: BigEndian::read_u16(&bytes[(port_start + 2)..(port_start + 4)]),
    })
}

impl Flow {
    #[inline]
    pub fn reverse_flow(&self) -> Flow {
        Flow {
            src_ip: self.dst_ip,
            dst_ip: self.src_ip,
            src_port: self.dst_port,
            dst_port: self.src_port,
            proto: self.proto,
        }
    }

    #[inline]
    pub fn ipv4_stamp_flow(&self, bytes: &mut [u8]) {
        let port_start = (bytes[0] & 0xf) as usize * IHL_TO_BYTE_FACTOR;
        BigEndian::write_u32(&mut bytes[12..16], self.src_ip);
        BigEndian::write_u32(&mut bytes[16..20], self.dst_ip);
        BigEndian::write_u16(&mut bytes[(port_start)..(port_start + 2)], self.src_port);
        BigEndian::write_u16(
            &mut bytes[(port_start + 2)..(port_start + 4)],
            self.dst_port,
        );
        BigEndian::write_u16(&mut bytes[10..12], 0);
        let csum = ipcsum(bytes);
        BigEndian::write_u16(&mut bytes[10..12], csum);
        // FIXME: l4 cksum
    }
}

/// Given the MAC payload, generate a flow hash. The flow hash generated depends on the IV, so different IVs will
/// produce different results (in cases when implementing Cuckoo hashing, etc.).
#[inline]
pub fn ipv4_flow_hash(bytes: &[u8], _iv: u32) -> usize {
    if let Some(flow) = ipv4_extract_flow(bytes) {
        flow_hash(&flow)
    } else {
        0
    }
}

#[inline]
pub fn flow_hash(flow: &Flow) -> usize {
    let mut hasher = FnvHasher::default();
    hasher.write(flow_as_u8(flow));
    hasher.finish() as usize
    // farmhash::hash32(flow_as_u8(flow))
}

/// Compute the CRC32 hash for `to_hash`. Note CRC32 is not really a great hash function, it is not particularly
/// collision resistant, and when implemented using normal instructions it is not particularly efficient. However, on
/// Intel processor's with SSE 4.2 and beyond, CRC32 is implemented in hardware, making it a bit faster than other
/// things, and is also what DPDK supports. Hence we use it here.
#[cfg_attr(feature = "dev", allow(inline_always))]
#[inline(always)]
pub fn crc_hash<T: Sized>(to_hash: &T, iv: u32) -> u32 {
    let size = mem::size_of::<T>();
    unsafe {
        let to_hash_bytes = (to_hash as *const T) as *const u8;
        crc_hash_native(to_hash_bytes, size as u32, iv)
    }
}

fn flow_as_u8(flow: &Flow) -> &[u8] {
    let size = mem::size_of::<Flow>();
    unsafe { slice::from_raw_parts((flow as *const Flow) as *const u8, size) }
}

#[inline]
fn ipcsum(payload: &[u8]) -> u16 {
    unsafe { ipv4_cksum(payload.as_ptr()) }
}
