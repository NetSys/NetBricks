use colored::*;
use netbricks::common::Result;
use netbricks::packets::ip::v4::Ipv4;
use netbricks::packets::ip::v6::Ipv6;
use netbricks::packets::ip::IpPacket;
use netbricks::packets::{Ethernet, Packet, RawPacket, Tcp};

#[inline]
pub fn eth_nf(packet: RawPacket) -> Result<Ethernet> {
    let ethernet = packet.parse::<Ethernet>()?;

    let info_fmt = format!("[eth] {}", ethernet).magenta().bold();
    println!("{}", info_fmt);

    Ok(ethernet)
}

#[inline]
pub fn ipv4_nf(ethernet: Ethernet) -> Result<Ethernet> {
    let ipv4 = ethernet.parse::<Ipv4>()?;
    let info_fmt = format!("[ipv4] {}, [offset] {}", ipv4, ipv4.offset()).yellow();
    println!("{}", info_fmt);

    let tcp = ipv4.parse::<Tcp<Ipv4>>()?;
    print_tcp(&tcp);

    Ok(tcp.deparse().deparse())
}

#[inline]
pub fn ipv6_nf(ethernet: Ethernet) -> Result<Ethernet> {
    let ipv6 = ethernet.parse::<Ipv6>()?;
    let info_fmt = format!("[ipv6] {}, [offset] {}", ipv6, ipv6.offset()).cyan();
    println!("{}", info_fmt);

    let tcp = ipv6.parse::<Tcp<Ipv6>>()?;
    print_tcp(&tcp);

    Ok(tcp.deparse().deparse())
}

#[inline]
fn print_tcp<T: IpPacket>(tcp: &Tcp<T>) {
    let tcp_fmt = format!("[tcp] {}", tcp).green();
    println!("{}", tcp_fmt);

    let flow_fmt = format!("[flow] {}", tcp.flow()).bright_blue();
    println!("{}", flow_fmt);
}
