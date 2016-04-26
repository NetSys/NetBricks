use byteorder::{BigEndian, ByteOrder};
use fnv::FnvHasher;
use headers::MacHeader;

use std::hash::Hasher;
use std::mem;
use std::slice;

// FIXME: Currently just deriving Hash, but figure out if this is a performance problem. By default, Rust uses SipHash
// which is supposed to have reasonable performance characteristics.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(C,packed)]
pub struct Flow {
    pub src_ip: u32,
    pub dst_ip: u32,
    pub src_port: u16,
    pub dst_port: u16,
    pub proto: u8,
}

const IHL_TO_BYTE_FACTOR: usize = 4; // IHL is in terms of number of 32-bit words.

/// This assumes the function is given the Mac Payload
#[inline]
pub fn ipv4_extract_flow(header: &MacHeader, bytes: &[u8]) -> Option<Flow> {
    if header.etype() == 0x0800 {
        let port_start = (bytes[0] & 0xf) as usize * IHL_TO_BYTE_FACTOR;
        Some(Flow {
            proto: bytes[9],
            src_ip: BigEndian::read_u32(&bytes[12..16]),
            dst_ip: BigEndian::read_u32(&bytes[16..20]),
            src_port: BigEndian::read_u16(&bytes[(port_start)..(port_start + 2)]),
            dst_port: BigEndian::read_u16(&bytes[(port_start + 2)..(port_start + 4)]),
        })
    } else {
        None
    }
}

/// Given the MAC payload, generate a flow hash. The flow hash generated depends on the IV, so different IVs will
/// produce different results (in cases when implementing Cuckoo hashing, etc.).
#[inline]
pub fn ipv4_flow_hash(header: &MacHeader, bytes: &[u8], _iv: u32) -> usize {
    if let Some(flow) = ipv4_extract_flow(header, bytes) {
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

#[link(name = "zcsi")]
extern "C" {
    fn crc_hash_native(to_hash: *const u8, size: u32, iv: u32) -> u32;
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
    unsafe { slice::from_raw_parts(((flow as *const Flow) as *const u8), size) }
}
