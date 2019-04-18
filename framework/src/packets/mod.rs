use common::Result;
use failure::Fail;
use native::zcsi::MBuf;

pub use self::ethernet::*;
pub use self::raw::*;
pub use self::tcp::*;
pub use self::udp::*;

pub mod buffer;
pub mod checksum;
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

    /// Returns a reference to the encapsulating packet
    fn envelope(&self) -> &Self::Envelope;

    /// Returns a mutable reference to the encapsulating packet
    fn envelope_mut(&mut self) -> &mut Self::Envelope;

    /// Returns a pointer to the DPDK message buffer
    #[doc(hidden)]
    fn mbuf(&self) -> *mut MBuf;

    /// Returns the buffer offset where the packet header begins
    fn offset(&self) -> usize;

    /// Returns a reference to the packet header
    #[doc(hidden)]
    fn header(&self) -> &Self::Header;

    /// Returns a mutable reference to the packet header
    #[doc(hidden)]
    fn header_mut(&mut self) -> &mut Self::Header;

    /// Returns the length of the packet header
    ///
    /// Includes both the fixed and variable portion of the header
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

    /// Parses the payload as packet of `T`
    #[inline]
    fn parse<T: Packet<Envelope = Self>>(self) -> Result<T>
    where
        Self: Sized,
    {
        T::do_parse(self)
    }

    // the public `parse::<T>` delegates to this function
    #[doc(hidden)]
    fn do_parse(envelope: Self::Envelope) -> Result<Self>
    where
        Self: Sized;

    /// Pushes a new packet `T` as the payload
    #[inline]
    fn push<T: Packet<Envelope = Self>>(self) -> Result<T>
    where
        Self: Sized,
    {
        T::do_push(self)
    }

    // the public `push::<T>` delegates to this function
    #[doc(hidden)]
    fn do_push(envelope: Self::Envelope) -> Result<Self>
    where
        Self: Sized;

    /// Removes this packet's header from the message buffer
    ///
    /// The packet's payload becomes the payload of its envelope. The
    /// result of the removal is not guaranteed to be a valid packet.
    fn remove(self) -> Result<Self::Envelope>
    where
        Self: Sized;

    /// Cascades the changes recursively through the layers
    ///
    /// An upper layer change to message buffer size can have cascading
    /// effects on a lower layer packet header. This call recursively ensures
    /// such changes are propogated through all the layers.
    fn cascade(&mut self);

    /// Deparses the packet and returns its envelope
    fn deparse(self) -> Self::Envelope;

    /// Resets the parsed packet back to raw packet
    fn reset(self) -> RawPacket
    where
        Self: Sized,
    {
        RawPacket::from_mbuf(self.mbuf())
    }
}

/// Error when packet failed to parse
#[derive(Debug, Fail)]
#[fail(display = "{}", _0)]
pub struct ParseError(String);

impl ParseError {
    fn new(msg: &str) -> ParseError {
        ParseError(msg.into())
    }
}
