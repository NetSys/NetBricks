use common::{Result, NetBricksError};
use failure::Fail;
use native::zcsi::MBuf;

pub use self::ethernet::*;
pub use self::raw::*;

pub mod ethernet;
pub mod icmp;
pub mod ip;
pub mod raw;

/// Fixed packet header
/// 
/// Some packet headers are variable in length, such as the IPv6 
/// segment routing header. The fixed portion can be statically 
/// defined, but the variable portion has to be parsed separately.
pub trait Header {
    /// Returns the size of the fixed header in bytes
    fn size() -> usize;
}

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
        header: *mut Self::Header) -> Self;

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

    /// Casts data at offset as a mutable reference T
    #[inline]
    fn get_mut_item<T>(&self, offset: usize) -> *mut T {
        unsafe {
            (*self.mbuf()).data_address(offset) as *mut T
        }
    }

    /// Parses the packet payload as another packet
    #[inline]
    fn parse<T: Packet<Envelope=Self>>(self) -> Result<T> where Self: std::marker::Sized {
        if self.payload_len() >= T::Header::size() {
            let mbuf = self.mbuf();
            let offset = self.payload_offset();
            let header = self.get_mut_item::<T::Header>(offset);
            Ok(T::from_packet(self, mbuf, offset, header))
        } else {
            Err(NetBricksError::BadOffset(self.payload_offset()).into())
        }
    }
}

/// Error when the packet does not match the expected type
#[derive(Fail, Debug)]
#[fail(display = "The packet is not {}", packet_type)]
pub struct ParseError {
    pub packet_type: String
}
