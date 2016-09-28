use e2d2::headers::*;
use e2d2::operators::*;

#[inline]
pub fn chain_nf<T: 'static + Batch<Header = NullHeader>>(parent: T) -> CompositionBatch<IpHeader> {
    parent.parse::<MacHeader>()
        .transform(box move |pkt| {
            let mut hdr = pkt.get_mut_header();
            hdr.swap_addresses();
        })
        .parse::<IpHeader>()
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
pub fn chain<T: 'static + Batch<Header = NullHeader>>(parent: T, len: u32) -> CompositionBatch<IpHeader> {
    let mut chained = chain_nf(parent);
    for _ in 1..len {
        chained = chain_nf(chained);
    }
    chained
}
