use common::{Result, NetBricksError};
use native::zcsi::MBuf;

pub use self::ethernet::*;
pub use self::raw::*;

pub mod ethernet;
pub mod ip;
pub mod raw;

/// trait all headers implement
pub trait Header {
    /// size of the header struct
    fn size() -> usize;
}

/// trait all packets implement
pub trait Packet {
    type Header: Header;
    type PreviousPacket: Packet;

    /// packet from the previous packet
    fn from_packet(
        previous: Self::PreviousPacket,
        mbuf: *mut MBuf,
        offset: usize,
        header: *mut Self::Header) -> Self;

    /// reference to the underlying mbuf
    fn mbuf(&self) -> *mut MBuf;

    /// offset where the packet starts
    fn offset(&self) -> usize;

    /// mutable reference to the packet header
    fn header(&self) -> &mut Self::Header;

    /// the length of the packet header
    fn header_len(&self) -> usize;

    /// length of the packet
    #[inline]
    fn len(&self) -> usize {
        unsafe { (*self.mbuf()).data_len() - self.offset() }
    }

    #[inline]
    /// offset where the packet payload starts
    fn payload_offset(&self) -> usize {
        self.offset() + self.header_len()
    }

    #[inline]
    /// length of the payload
    fn payload_len(&self) -> usize {
        self.len() - self.header_len()
    }

    /// extends the packet by n bytes
    #[inline]
    fn extend(&self, extend_by: usize) -> Result<()> {
        unsafe {
            match (*self.mbuf()).add_data_end(extend_by) {
                0 => Err(NetBricksError::FailedAllocation.into()),
                _ => Ok(())
            }
        }
    }

    /// get a mutable reference to T at offset
    #[inline]
    fn get_mut_item<T>(&self, offset: usize) -> *mut T {
        unsafe {
            (*self.mbuf()).data_address(offset) as *mut T
        }
    }

    /// parse the payload as the next packet
    #[inline]
    fn parse<T: Packet<PreviousPacket=Self>>(self) -> Result<T> where Self: std::marker::Sized {
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
