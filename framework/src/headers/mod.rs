pub use self::ip::*;
pub use self::mac::*;
pub use self::null_header::*;
pub use self::tcp::*;
pub use self::udp::*;
pub mod ip;
pub mod mac;

mod null_header;
mod tcp;
mod udp;

// L4 Protocol Next Header Values
pub const TCP_NXT_HDR: u8 = 6;
pub const UDP_NXT_HDR: u8 = 17;
pub const ICMP_NXT_HDR: u8 = 1;


#[derive(FromPrimitive, Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum L4Protocol {
    Tcp = TCP_NXT_HDR,
    Udp = UDP_NXT_HDR,
    Icmp = ICMP_NXT_HDR
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
    fn payload_size(&self, hint: usize) -> usize;

    fn check_correct(&self, prev: &Self::PreviousHeader) -> bool;
}

/// A trait implemented on headers that provide updates on byte-changes to packets
/// TODO: Eventually roll this and other setters into packet actions like remove,
///       insert, swap, etc, as part of specific changes to certain *types* of
///       headers in a packet.
///       In ref. to https://github.comcast.com/occam/og/pull/103#discussion_r293652
pub trait HeaderUpdates {
    fn update_payload_len(&mut self, payload_diff: isize);
    fn update_next_header(&mut self, hdr: NextHeader);
}
