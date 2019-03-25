use common::{Result, NetBricksError};
use failure::Fail;
use native::zcsi::MBuf;

pub use self::ethernet::*;
pub use self::raw::*;
pub use self::tcp::*;
pub use self::udp::*;

pub mod buffer;
pub mod ethernet;
pub mod icmp;
pub mod ip;
pub mod raw;
pub mod tcp;
pub mod udp;

/// Type that has a fixed size
/// 
/// Size of the structs are used for buffer bound check when parsing packets
pub trait Fixed {
    /// Returns the size of the type
    fn size() -> usize;
}

impl<T> Fixed for T {
    #[inline]
    fn size() -> usize {
        std::mem::size_of::<T>()
    }
}

/// Fixed packet header marker trait
/// 
/// Some packet headers are variable in length, such as the IPv6 
/// segment routing header. The fixed portion can be statically 
/// defined, but the variable portion has to be parsed separately.
pub trait Header: Fixed {}

/// Common behaviors shared by all packets
pub trait Packet {
    /// The header type of the packet
    type Header: Header;
    /// The outer packet type that encapsulates the packet
    type Envelope: Packet;

    /// Creates a new packet
    fn from_packet(
        envelope: Self::Envelope,
        mbuf: *mut MBuf,
        offset: usize,
        header: *mut Self::Header) -> Result<Self> where Self: Sized;

    /// Returns the packet that encapsulated this packet
    fn envelope(&self) -> &Self::Envelope;

    /// Returns the DPDK buffer
    fn mbuf(&self) -> *mut MBuf;

    /// Returns the buffer offset where the packet header begins
    fn offset(&self) -> usize;

    /// Returns a mutable reference to the packet header
    fn header(&self) -> &mut Self::Header;

    /// Returns the length of the packet header
    fn header_len(&self) -> usize;

    /// Returns the length of the packet
    #[inline]
    fn len(&self) -> usize {
        unsafe { (*self.mbuf()).data_len() - self.offset() }
    }

    /// Returns the buffer offset where the packet payload begins
    #[inline]
    fn payload_offset(&self) -> usize {
        self.offset() + self.header_len()
    }

    /// Returns the length of the packet payload
    #[inline]
    fn payload_len(&self) -> usize {
        self.len() - self.header_len()
    }

    /// Extends the end of the packet buffer by n bytes
    #[inline]
    fn extend(&self, extend_by: usize) -> Result<()> {
        unsafe {
            match (*self.mbuf()).add_data_end(extend_by) {
                0 => Err(NetBricksError::FailedAllocation.into()),
                _ => Ok(())
            }
        }
    }

    /// Parses the packet payload as another packet
    #[inline]
    fn parse<T: Packet<Envelope=Self>>(self) -> Result<T> where Self: std::marker::Sized {
        let mbuf = self.mbuf();
        let offset = self.payload_offset();
        let header = buffer::read_item::<T::Header>(mbuf, offset)?;
        T::from_packet(self, mbuf, offset, header)
    }
}

/// Error when packet failed to parse
#[derive(Fail, Debug)]
#[fail(display = "{}", _0)]
pub struct ParseError(String);

impl ParseError {
    fn new(msg: &str) -> ParseError {
        ParseError(msg.into())
    }
}
