extern crate byteorder;
use byteorder::{ByteOrder, BigEndian};

#[derive(Debug, Copy, Clone, Default)]
pub struct Flow {
    pub src_ip: u32,
    pub dst_ip: u32,
    pub src_port: u16,
    pub dst_port: u16,
    pub proto: u8,
}

/// This assumes the function is given the Mac Payload
pub fn ipv4_extract_flow(bytes: &[u8]) -> Flow {
    Flow {
        proto: bytes[9],
        src_ip: BigEndian::read_u32(&bytes[12..16]),
        dst_ip: BigEndian::read_u32(&bytes[16..20]),
        src_port: BigEndian::read_u16(&bytes[20..22]),
        dst_port: BigEndian::read_u16(&bytes[22..24]),
    }
}
