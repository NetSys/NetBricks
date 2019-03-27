use common::{NetBricksError, Result};
use native::zcsi::{mbuf_alloc, MBuf};
use packets::{buffer, Header, Packet};

/// Unit header
impl Header for () {}

/// The raw network packet
///
/// Simply a wrapper around the underlying buffer with packet semantic
pub struct RawPacket {
    mbuf: *mut MBuf,
}

impl RawPacket {
    /// Creates a new packet by allocating a new buffer
    pub fn new() -> Result<Self> {
        unsafe {
            let mbuf = mbuf_alloc();
            if mbuf.is_null() {
                Err(NetBricksError::FailedAllocation.into())
            } else {
                Ok(RawPacket { mbuf })
            }
        }
    }

    /// Creates a new packet and initialize the buffer with a byte array
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        let packet = RawPacket::new()?;
        buffer::alloc(packet.mbuf, 0, data.len())?;
        buffer::write_slice(packet.mbuf, 0, data)?;
        Ok(packet)
    }
}

impl Packet for RawPacket {
    type Header = ();
    type Envelope = RawPacket;

    #[inline]
    fn envelope(&self) -> &Self::Envelope {
        &self
    }

    #[inline]
    fn mbuf(&self) -> *mut MBuf {
        self.mbuf
    }

    #[inline]
    fn offset(&self) -> usize {
        0
    }

    #[inline]
    fn header(&self) -> &mut Self::Header {
        unreachable!("raw packet has no defined header!");
    }

    #[inline]
    fn header_len(&self) -> usize {
        0
    }

    #[doc(hidden)]
    #[inline]
    fn do_parse(envelope: Self::Envelope) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(envelope)
    }

    #[doc(hidden)]
    #[inline]
    fn do_push(envelope: Self::Envelope) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(envelope)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dpdk_test;

    #[test]
    fn new_raw_packet() {
        dpdk_test! {
            assert!(RawPacket::new().is_ok());
        }
    }

    #[test]
    fn raw_packet_from_bytes() {
        use packets::udp::tests::UDP_PACKET;

        dpdk_test! {
            assert!(RawPacket::from_bytes(&UDP_PACKET).is_ok());
        }
    }
}
