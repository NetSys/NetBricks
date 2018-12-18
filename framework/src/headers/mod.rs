pub use self::ip::*;
pub use self::mac::*;
pub use self::null_header::*;
pub use self::tcp::*;
pub use self::udp::*;
use std::net::Ipv6Addr;

pub mod ip;
pub mod mac;
mod null_header;
mod tcp;
mod udp;

// L3/L4 Protocol Next Header Values
pub const TCP_NXT_HDR: u8 = 6;
pub const UDP_NXT_HDR: u8 = 17;
pub const ICMP_NXT_HDR: u8 = 58;

// MTU
// const MTU: u32 = 1500;
// const MAX_PKT_SIZE: u32 = MTU + 14; // 1500 ip hdr size + 14 ethernet hdr size

#[derive(FromPrimitive, Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum Protocol {
    Tcp = TCP_NXT_HDR,
    Udp = UDP_NXT_HDR,
    Icmp = ICMP_NXT_HDR,
}

/// A trait implemented by all headers, used for reading them from a mbuf.
pub trait EndOffset: Send {
    type PreviousHeader: EndOffset;

    /// Offset returns the number of bytes to skip to get to the next header, relative to the start
    /// of the mbuf.
    fn offset(&self) -> usize;

    /// Returns the size of this header in bytes.
    fn size() -> usize;

    /// Returns the size of the payload in bytes. The hint is necessary for things like the L2 header which have no
    /// explicit length field.
    fn payload_size(&self, _hint: usize) -> usize;

    /// Apply a default here until we use it.
    fn check_correct(&self, _prev: &Self::PreviousHeader) -> bool {
        true
    }
}

pub trait CalcChecksums {
    fn checksum(&self) -> u16;
    fn set_checksum(&mut self, csum: u16);

    fn update_v6_checksum(
        &mut self,
        segment_length: u32,
        src: Ipv6Addr,
        dst: Ipv6Addr,
        proto: Protocol,
    ) {
        let mut sum = segment_length
            + src.segments().iter().map(|x| *x as u32).sum::<u32>()
            + dst.segments().iter().map(|x| *x as u32).sum::<u32>()
            + proto as u32;

        while sum >> 16 != 0 {
            sum = (sum >> 16) + (sum & 0xFFFF);
        }

        self.set_checksum(!sum as u16)
    }

    /*
    see https://tools.ietf.org/html/rfc1624
    essentially, if HC is the original checksum, m is original 16 bit word
    of the header, HC', the new header checksum, m' the new 16 bit word then
    HC' = ~(~HC + ~m + m')
    */
    fn update_v6_checksum_incremental(
        &mut self,
        old_field: Ipv6Addr,
        updated_field: Ipv6Addr,
    ) -> Option<u16> {
        let mut old_checksum = self.checksum();
        if old_checksum == 0 {
            return None;
        }

        let old_segments = old_field.segments();
        let updated_segments = updated_field.segments();
        let mut sum = 0;

        for i in 0..updated_segments.len() {
            let old_frag = old_segments[i] & 0xFFFF;
            let updated_frag = updated_segments[i] & 0xFFFF;

            match ((!old_checksum & 0xFFFF) as u32)
                .checked_add((!old_frag & 0xFFFF) as u32 + (updated_frag & 0xFFFF) as u32)
            {
                Some(added) => sum = added,
                None => return None,
            }

            sum = (sum >> 16 & 0xFFFF) + (sum & 0xFFFF);
            sum = !sum & 0xFFFF;
            old_checksum = sum as u16
        }

        let fin_sum = sum as u16;

        self.set_checksum(fin_sum);
        Some(fin_sum)
    }
}

/// A trait implemented on headers that provide updates on byte-changes to packets
/// TODO: Eventually roll this and other setters into packet actions like remove,
///       insert, swap, etc, as part of specific changes to certain *types* of
///       headers in a packet.
///       In ref. to https://github.comcast.com/occam/og/pull/103#discussion_r293652
pub trait HeaderUpdates {
    type PreviousHeader: EndOffset;

    fn update_payload_len(&mut self, payload_diff: isize);
    fn update_next_header(&mut self, hdr: NextHeader);
}
