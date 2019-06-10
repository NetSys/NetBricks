pub mod v4;
pub mod v6;

use crate::common::Result;
use crate::packets::checksum::PseudoHeader;
use crate::packets::Packet;
use failure::Fail;
use std::fmt;
use std::net::IpAddr;

/// Assigned internet protocol number
///
/// From https://www.iana.org/assignments/protocol-numbers/protocol-numbers.xhtml
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C, packed)]
pub struct ProtocolNumber(pub u8);

impl ProtocolNumber {
    pub fn new(value: u8) -> Self {
        ProtocolNumber(value)
    }
}

/// Supported protocol numbers
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub mod ProtocolNumbers {
    use super::ProtocolNumber;

    // Transmission Control Protocol
    pub const Tcp: ProtocolNumber = ProtocolNumber(0x06);

    // User Datagram Protocol
    pub const Udp: ProtocolNumber = ProtocolNumber(0x11);

    // Routing Header for IPv6
    pub const Ipv6Route: ProtocolNumber = ProtocolNumber(0x2B);

    // Internet Control Message Protocol for IPv6
    pub const Icmpv6: ProtocolNumber = ProtocolNumber(0x3A);

    // Internet Control Message Protocol for IPv4
    pub const Icmpv4: ProtocolNumber = ProtocolNumber(0x01);
}

impl fmt::Display for ProtocolNumber {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                ProtocolNumbers::Tcp => "TCP".to_string(),
                ProtocolNumbers::Udp => "UDP".to_string(),
                ProtocolNumbers::Ipv6Route => "IPv6 Route".to_string(),
                ProtocolNumbers::Icmpv6 => "ICMPv6".to_string(),
                _ => format!("0x{:02x}", self.0),
            }
        )
    }
}

/// Common behaviors shared by IPv4 and IPv6 packets
pub trait IpPacket: Packet {
    /// Returns the assigned protocol number of the header immediately follows
    ///
    /// For IPv4 headers, this should be the `protocol` field.
    /// For IPv6 and extension headers, this should be the `next header` field.
    fn next_proto(&self) -> ProtocolNumber;

    /// Returns the source IP address
    fn src(&self) -> IpAddr;

    /// Sets the source IP address
    ///
    /// This lets an upper layer packet like TCP set the source IP address
    /// on a lower layer packet.
    fn set_src(&mut self, src: IpAddr) -> Result<()>;

    /// Returns the destination IP address
    fn dst(&self) -> IpAddr;

    /// Sets the destination IP address
    ///
    /// This lets an upper layer packet like TCP set the destination IP address
    /// on a lower layer packet.
    fn set_dst(&mut self, dst: IpAddr) -> Result<()>;

    /// Returns the pseudo-header for layer 4 checksum computation
    fn pseudo_header(&self, packet_len: u16, protocol: ProtocolNumber) -> PseudoHeader;
}

/// 5-tuple IP connection identifier
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Flow {
    src_ip: IpAddr,
    dst_ip: IpAddr,
    src_port: u16,
    dst_port: u16,
    protocol: ProtocolNumber,
}

impl Flow {
    pub fn new(
        src_ip: IpAddr,
        dst_ip: IpAddr,
        src_port: u16,
        dst_port: u16,
        protocol: ProtocolNumber,
    ) -> Self {
        Flow {
            src_ip,
            dst_ip,
            src_port,
            dst_port,
            protocol,
        }
    }

    #[inline]
    pub fn src_ip(&self) -> IpAddr {
        self.src_ip
    }

    #[inline]
    pub fn set_src_ip(&mut self, src_ip: IpAddr) {
        self.src_ip = src_ip
    }

    #[inline]
    pub fn dst_ip(&self) -> IpAddr {
        self.dst_ip
    }

    #[inline]
    pub fn set_dst_ip(&mut self, dst_ip: IpAddr) {
        self.dst_ip = dst_ip
    }

    #[inline]
    pub fn src_port(&self) -> u16 {
        self.src_port
    }

    #[inline]
    pub fn set_src_port(&mut self, src_port: u16) {
        self.src_port = src_port
    }

    #[inline]
    pub fn dst_port(&self) -> u16 {
        self.dst_port
    }

    #[inline]
    pub fn set_dst_port(&mut self, dst_port: u16) {
        self.dst_port = dst_port
    }

    #[inline]
    pub fn protocol(&self) -> ProtocolNumber {
        self.protocol
    }

    #[inline]
    pub fn set_protocol(&mut self, protocol: ProtocolNumber) {
        self.protocol = protocol
    }

    #[inline]
    pub fn reverse(&self) -> Self {
        Flow {
            src_ip: self.dst_ip,
            dst_ip: self.src_ip,
            src_port: self.dst_port,
            dst_port: self.src_port,
            protocol: self.protocol,
        }
    }
}

impl fmt::Display for Flow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "src_ip: {}, src_port: {}, dst_ip: {}, dst_port: {}, proto: {}",
            self.src_ip(),
            self.src_port(),
            self.dst_ip(),
            self.dst_port(),
            self.protocol()
        )
    }
}

#[derive(Debug, Fail)]
#[fail(display = "Cannot mix IPv4 and IPv6 addresses")]
pub struct IpAddrMismatchError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_number_to_string() {
        assert_eq!("TCP", ProtocolNumbers::Tcp.to_string());
        assert_eq!("UDP", ProtocolNumbers::Udp.to_string());
        assert_eq!("IPv6 Route", ProtocolNumbers::Ipv6Route.to_string());
        assert_eq!("ICMPv6", ProtocolNumbers::Icmpv6.to_string());
        assert_eq!("0x00", ProtocolNumber::new(0).to_string());
    }
}
