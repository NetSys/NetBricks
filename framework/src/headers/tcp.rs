use super::EndOffset;
use headers::*;
use std::default::Default;
use std::fmt;
use std::marker::PhantomData;

#[repr(C, packed)]
pub struct TcpHeader<T> {
    src_port: u16,
    dst_port: u16,
    seq: u32,
    ack: u32,
    offset_to_ns: u8,
    flags: u8,
    window: u16,
    csum: u16,
    urgent: u16,
    _parent: PhantomData<T>,
}

const CWR: u8 = 0b1000_0000;
const ECE: u8 = 0b0100_0000;
const URG: u8 = 0b0010_0000;
const ACK: u8 = 0b0001_0000;
const PSH: u8 = 0b0000_1000;
const RST: u8 = 0b0000_0100;
const SYN: u8 = 0b0000_0010;
const FIN: u8 = 0b0000_0001;

macro_rules! write_or_return {
    ($dst: expr, $($arg:tt)*) => {
        {
            let result = write!($dst, $($arg)*);
            if result.is_err() {
                return result;
            }
        }
    }
}

impl<T> Default for TcpHeader<T>
where
    T: IpHeader,
{
    fn default() -> TcpHeader<T> {
        TcpHeader {
            src_port: 0,
            dst_port: 0,
            seq: 0,
            ack: 0,
            offset_to_ns: 0,
            flags: 0,
            window: 0,
            csum: 0,
            urgent: 0,
            _parent: PhantomData,
        }
    }
}

impl<T> fmt::Display for TcpHeader<T>
where
    T: IpHeader,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write_or_return!(
            f,
            "tcp src_port {} dst_port {} seq {} ack {} data_offset {} flags ",
            self.src_port(),
            self.dst_port(),
            self.seq_num(),
            self.ack_num(),
            self.data_offset()
        );
        let ret = self.fmt_flags(f);
        if ret.is_err() {
            return ret;
        }
        write!(
            f,
            "cwnd {} csum {} urgent {}",
            self.window_size(),
            self.checksum(),
            self.urgent()
        )
    }
}

impl<T> EndOffset for TcpHeader<T>
where
    T: IpHeader,
{
    type PreviousHeader = T;

    #[inline]
    fn offset(&self) -> usize {
        (self.data_offset() * 4) as usize
    }

    #[inline]
    fn size() -> usize {
        20
    }

    #[inline]
    fn payload_size(&self, frame_size: usize) -> usize {
        frame_size - self.offset()
    }

    #[inline]
    fn check_correct(&self, _prev: &T) -> bool {
        true
    }
}

impl<T> CalcChecksums for TcpHeader<T>
where
    T: IpHeader,
{
    #[inline]
    fn checksum(&self) -> u16 {
        u16::from_be(self.csum)
    }

    #[inline]
    fn set_checksum(&mut self, csum: u16) {
        self.csum = u16::to_be(csum)
    }
}

impl<T> TcpHeader<T>
where
    T: IpHeader,
{
    #[inline]
    pub fn new() -> TcpHeader<T> {
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
    pub fn seq_num(&self) -> u32 {
        u32::from_be(self.seq)
    }

    #[inline]
    pub fn set_seq_num(&mut self, seq: u32) {
        self.seq = u32::to_be(seq);
    }

    #[inline]
    pub fn ack_num(&self) -> u32 {
        u32::from_be(self.ack)
    }

    #[inline]
    pub fn set_ack_num(&mut self, ack: u32) {
        self.ack = u32::to_be(ack);
    }

    #[inline]
    pub fn data_offset(&self) -> u8 {
        (self.offset_to_ns & 0xf0) >> 4
    }

    #[inline]
    pub fn set_data_offset(&mut self, offset: u8) {
        self.offset_to_ns = (self.offset_to_ns & 0x0f) | (offset << 4);
    }

    // BEGIN FLAGS
    // This is weird in being located outside the flags byte.
    /// ECN nonce concealment flag: RFC 3540.
    #[inline]
    pub fn ns_flag(&self) -> bool {
        (self.offset_to_ns & 0x01) == 1
    }

    #[inline]
    pub fn set_ns(&mut self) {
        self.offset_to_ns |= 1
    }

    #[inline]
    pub fn unset_ns(&mut self) {
        self.offset_to_ns &= !0x1
    }

    /// Congestion window reduction flag.
    // FIXME: Autogenerate after https://github.com/rust-lang/rust/issues/29599 is fixed.
    #[inline]
    pub fn cwr_flag(&self) -> bool {
        (self.flags & CWR) != 0
    }

    /// Set CWR flag to 1.
    #[inline]
    pub fn set_cwr_flag(&mut self) {
        self.flags |= CWR;
    }

    /// Set CWR flag to 0.
    #[inline]
    pub fn unset_cwr_flag(&mut self) {
        self.flags &= !CWR;
    }

    /// ECN echo flag.
    #[inline]
    pub fn ece_flag(&self) -> bool {
        (self.flags & ECE) != 0
    }

    /// Set ECE flag to 1.
    #[inline]
    pub fn set_ece_flag(&mut self) {
        self.flags |= ECE;
    }

    /// Set ECE flag to 0.
    #[inline]
    pub fn unset_ece_flag(&mut self) {
        self.flags &= !ECE;
    }

    /// Urgent pointer field is significant.
    #[inline]
    pub fn urg_flag(&self) -> bool {
        (self.flags & URG) != 0
    }

    /// Set URG flag to 1.
    #[inline]
    pub fn set_urg_flag(&mut self) {
        self.flags |= URG;
    }

    /// Set URG flag to 0.
    #[inline]
    pub fn unset_urg_flag(&mut self) {
        self.flags &= !URG;
    }

    /// Acknowledgment field is significant.
    #[inline]
    pub fn ack_flag(&self) -> bool {
        (self.flags & ACK) != 0
    }

    /// Set ACK flag to 1.
    #[inline]
    pub fn set_ack_flag(&mut self) {
        self.flags |= ACK;
    }

    /// Set ACK flag to 0.
    #[inline]
    pub fn unset_ack_flag(&mut self) {
        self.flags &= !ACK;
    }

    /// Push function: Push buffered data to receiving application.
    #[inline]
    pub fn psh_flag(&self) -> bool {
        (self.flags & PSH) != 0
    }

    /// Set PSH flag to 1.
    #[inline]
    pub fn set_psh_flag(&mut self) {
        self.flags |= PSH;
    }

    /// Set PSH flag to 0.
    #[inline]
    pub fn unset_psh_flag(&mut self) {
        self.flags &= !PSH;
    }

    /// Reset connection.
    #[inline]
    pub fn rst_flag(&self) -> bool {
        (self.flags & RST) != 0
    }

    /// Set RST flag to 1.
    #[inline]
    pub fn set_rst_flag(&mut self) {
        self.flags |= RST;
    }

    /// Set RST flag to 0.
    #[inline]
    pub fn unset_rst_flag(&mut self) {
        self.flags &= !RST;
    }

    /// Synchronize sequence number.
    #[inline]
    pub fn syn_flag(&self) -> bool {
        (self.flags & SYN) != 0
    }

    /// Set SYN flag to 1.
    #[inline]
    pub fn set_syn_flag(&mut self) {
        self.flags |= SYN;
    }

    /// Set SYN flag to 0.
    #[inline]
    pub fn unset_syn_flag(&mut self) {
        self.flags &= !SYN;
    }

    /// No more data transfer from sender.
    #[inline]
    pub fn fin_flag(&self) -> bool {
        (self.flags & FIN) != 0
    }

    /// Set FIN flag to 1.
    #[inline]
    pub fn set_fin_flag(&mut self) {
        self.flags |= FIN;
    }

    /// Set FIN flag to 0.
    #[inline]
    pub fn unset_fin_flag(&mut self) {
        self.flags &= !FIN;
    }

    pub fn fmt_flags(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write_or_return!(f, "| ");
        if self.ns_flag() {
            write_or_return!(f, "NS ")
        };
        if self.cwr_flag() {
            write_or_return!(f, "CWR ")
        };
        if self.ece_flag() {
            write_or_return!(f, "ECE ")
        };
        if self.urg_flag() {
            write_or_return!(f, "URG ")
        };
        if self.ack_flag() {
            write_or_return!(f, "ACK ")
        };
        if self.psh_flag() {
            write_or_return!(f, "PSH ")
        };
        if self.rst_flag() {
            write_or_return!(f, "RST ")
        };
        if self.syn_flag() {
            write_or_return!(f, "SYN ")
        };
        if self.fin_flag() {
            write_or_return!(f, "FIN ")
        };
        write!(f, "|")
    }
    // END FLAGS

    /// Receive window.
    pub fn window_size(&self) -> u16 {
        u16::from_be(self.window)
    }

    pub fn set_window_size(&mut self, wnd: u16) {
        self.window = u16::to_be(wnd);
    }

    /// Urgent pointer
    pub fn urgent(&self) -> u16 {
        u16::from_be(self.urgent)
    }

    pub fn set_urgent(&mut self, urgent: u16) {
        self.urgent = u16::to_be(urgent);
    }
}
