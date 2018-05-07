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

#[cfg(test)]
// Common testing helpers for headers
mod tests {
    use common::EmptyMetadata;
    use headers::null_header::NullHeader;
    use interface::{dpdk, new_packet, Packet};
    use std::sync::{Once, ONCE_INIT};

    static EAL_INIT: Once = ONCE_INIT;

    // Acquire a packet buffer for testing header extraction from raw bytes
    pub fn packet_from_bytes(bytes: &[u8]) -> Packet<NullHeader, EmptyMetadata> {
        EAL_INIT.call_once(|| {
            dpdk::init_system_wl("packet_overlay_tests", 0, &[]);
        });
        let mut pkt = new_packet().expect("Could not allocate packet!");
        pkt.increase_payload_size(bytes.len());
        {
            let payload = pkt.get_mut_payload();
            unsafe { bytes.as_ptr().copy_to(payload.as_mut_ptr(), bytes.len()) }
        }
        pkt
    }

}
