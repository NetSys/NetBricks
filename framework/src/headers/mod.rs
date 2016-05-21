pub use self::null_header::*;
pub use self::mac::*;
pub use self::ip::*;
pub use self::udp::*;
mod mac;
mod ip;
mod udp;
mod null_header;

/// A trait implemented by all headers, used for reading them from a mbuf.
pub trait EndOffset: Send {
    /// Offset returns the number of bytes to skip to get to the next header.
    fn offset(&self) -> usize;
    /// Returns the size of this header in bytes.
    fn size() -> usize;
    /// Returns the size of the payload in bytes. The hint is necessary for things like the L2 header which have no
    /// explicit length field.
    fn payload_size(&self, hint: usize) -> usize;
}
