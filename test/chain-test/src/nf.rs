use e2d2::common::EmptyMetadata;
use e2d2::headers::*;
use e2d2::operators::*;

#[inline]
pub fn chain_nf<T: 'static + Batch<Header = NullHeader, Metadata = EmptyMetadata>>(parent: T) -> CompositionBatch {
    parent
        .parse::<MacHeader>()
        .transform(box move |pkt| {
            let hdr = pkt.get_mut_header();
            hdr.swap_addresses();
        })
        .parse::<Ipv4Header>()
        .transform(box |pkt| {
            let h = pkt.get_mut_header();
            let ttl = h.ttl();
            h.set_ttl(ttl - 1);
        })
        .filter(box |pkt| {
            let h = pkt.get_header();
            h.ttl() != 0
        })
        .compose()
}

#[inline]
pub fn chain<T: 'static + Batch<Header = NullHeader, Metadata = EmptyMetadata>>(
    parent: T,
    len: u32,
    pos: u32,
) -> CompositionBatch {
    let mut chained = chain_nf(parent);
    for _ in 1..len {
        chained = chain_nf(chained);
    }
    if len % 2 == 0 || pos % 2 == 1 {
        chained
            .parse::<MacHeader>()
            .transform(box move |pkt| {
                let hdr = pkt.get_mut_header();
                hdr.swap_addresses();
            })
            .compose()
    } else {
        chained
    }
}
