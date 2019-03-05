use failure::Error;
use netbricks::headers::*;
use netbricks::operators::*;
use std::net::Ipv6Addr;
use std::str::FromStr;

fn throw_mama_from_the_train(hdr: &Ipv6Header) -> Result<(), Error> {
    let hextets = hdr.src().segments();
    let prefix = Ipv6Addr::new(hextets[0], hextets[1], hextets[2], hextets[3], 0, 0, 0, 0);
    if prefix == Ipv6Addr::from_str("da75::").unwrap() {
        bail!("directed by danny devito")
    } else {
        Ok(())
    }
}

pub fn nf<T: 'static + Batch<Header = NullHeader>>(
    parent: T,
) -> MapBatch<
    Ipv6Header,
    TransformResults<
        Ipv6Header,
        ParsedBatch<Ipv6Header, FilterBatch<MacHeader, ParsedBatch<MacHeader, T>>>,
    >,
> {
    parent
        .parse::<MacHeader>()
        .filter(box |pkt| match pkt.get_header().etype() {
            Some(EtherType::IPv6) => true,
            _ => false,
        })
        .parse::<Ipv6Header>()
        .transform_ok(box |pkt| {
            let v6 = pkt.get_header();
            throw_mama_from_the_train(v6)
        })
        .map(box |pkt| warn!("v6: {}", pkt.get_header()))
}
