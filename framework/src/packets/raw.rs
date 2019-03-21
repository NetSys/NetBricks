use common::{Result, NetBricksError};
use native::zcsi::{MBuf, mbuf_alloc};
use std::ptr;
use packets::{Packet, Header};

/// Unit header
impl Header for () {
    fn size() -> usize {
        0
    }
}

/// The raw network packet
///
/// Simply a wrapper around the underlying buffer with packet semantic
pub struct RawPacket {
    mbuf: *mut MBuf
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
        packet.extend(data.len())?;
        unsafe {
            ptr::copy_nonoverlapping(
                &data[0] as *const u8,
                (*packet.mbuf).data_address(0) as *mut u8,
                data.len()
            )
        }
        Ok(packet)
    }
}

impl Packet for RawPacket {
    type Header = ();
    type Envelope = RawPacket;

    #[inline]
    fn from_packet(envelope: Self::Envelope,
                   _mbuf: *mut MBuf,
                   _offset: usize,
                   _header: *mut Self::Header) -> Self {
        envelope
    }

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
        unsafe {
            &mut (*self.get_mut_item::<Self::Header>(0))
        }
    }

    #[inline]
    fn header_len(&self) -> usize {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dpdk_test;

    #[rustfmt::skip]
    const UDP_PACKET: [u8; 52] = [
        // ** ethernet header
        0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
        0x08, 0x00,
        // ** IPv4 header
        0x45, 0x00,
        // payload length
        0x00, 0x26,
        0xab, 0x49, 0x40, 0x00,
        0xff, 0x11, 0xf7, 0x00,
        0x8b, 0x85, 0xd9, 0x6e,
        0x8b, 0x85, 0xe9, 0x02,
        // ** UDP header
        0x99, 0xd0, 0x04, 0x3f,
        0x00, 0x12, 0x72, 0x28,
        // ** UDP payload
        0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x68, 0x65, 0x6c, 0x6c, 0x6f
    ];

    #[test]
    fn new_raw_packet() {
        dpdk_test! {
            assert!(RawPacket::new().is_ok());
        }
    }

    #[test]
    fn raw_packet_from_bytes() {
        dpdk_test! {
            assert!(RawPacket::from_bytes(&UDP_PACKET).is_ok());
        }
    }

    #[test]
    fn extend_packet() {
        dpdk_test! {
            let packet = RawPacket::new().unwrap();
            packet.extend(200).unwrap();
            assert_eq!(packet.len(), 200);
        }
    }

    #[test]
    fn exceed_mbuf_tailroom() {
        dpdk_test! {
            let packet = RawPacket::new().unwrap();
            assert!(packet.extend(999999).is_err());
        }
    }
}
