use super::EndOffset;
use headers::ip::IpHeader;
use std::default::Default;
use std::fmt;
use std::marker::PhantomData;

/// UDP header using SSE
#[repr(C, packed)]
pub struct UdpHeader<T> {
    src_port: u16,
    dst_port: u16,
    len: u16,
    csum: u16,
    _parent: PhantomData<T>,
}

impl<T> Default for UdpHeader<T>
where
    T: IpHeader,
{
    fn default() -> UdpHeader<T> {
        UdpHeader {
            src_port: 0,
            dst_port: 0,
            len: 0,
            csum: 0,
            _parent: PhantomData,
        }
    }
}

impl<T> fmt::Display for UdpHeader<T>
where
    T: IpHeader,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "src_port: {} dst_port: {} len: {} checksum: {}",
            self.src_port(),
            self.dst_port(),
            self.length(),
            self.checksum()
        )
    }
}

impl<T> EndOffset for UdpHeader<T>
where
    T: IpHeader,
{
    type PreviousHeader = T;
    #[inline]
    fn offset(&self) -> usize {
        8 // 8 bytes
    }

    #[inline]
    fn size() -> usize {
        8
    }

    #[inline]
    fn payload_size(&self, _: usize) -> usize {
        self.length() as usize - self.offset()
    }

    #[inline]
    fn check_correct(&self, _prev: &T) -> bool {
        true
    }
}

impl<T> UdpHeader<T>
where
    T: IpHeader,
{
    #[inline]
    pub fn new() -> UdpHeader<T> {
        Default::default()
    }

    #[inline]
    pub fn src_port(&self) -> u16 {
        u16::from_be(self.src_port)
    }

    #[inline]
    pub fn dst_port(&self) -> u16 {
        u16::from_be(self.dst_port)
    }

    #[inline]
    pub fn set_src_port(&mut self, port: u16) {
        self.src_port = u16::to_be(port);
    }

    #[inline]
    pub fn set_dst_port(&mut self, port: u16) {
        self.dst_port = u16::to_be(port);
    }

    #[inline]
    pub fn length(&self) -> u16 {
        u16::from_be(self.len)
    }

    #[inline]
    pub fn set_length(&mut self, len: u16) {
        self.len = u16::to_be(len)
    }

    #[inline]
    pub fn checksum(&self) -> u16 {
        u16::from_be(self.csum)
    }

    #[inline]
    pub fn set_checksum(&mut self, csum: u16) {
        self.csum = u16::to_be(csum);
    }
}
