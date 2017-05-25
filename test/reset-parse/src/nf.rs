use e2d2::headers::*;
use e2d2::operators::*;

#[inline]
fn lat() {
    unsafe {
        asm!("nop"
             :
             :
             :
             : "volatile");
    }
}

#[inline]
fn delay_loop(delay: u64) {
    let mut d = 0;
    while d < delay {
        lat();
        d += 1;
    }
}

pub fn delay<T: 'static + Batch<Header = NullHeader>>
    (parent: T,
     delay: u64)
     -> MapBatch<NullHeader, ResetParsingBatch<TransformBatch<MacHeader, ParsedBatch<MacHeader, T>>>> {
    parent
        .parse::<MacHeader>()
        .transform(box move |pkt| {
                           assert!(pkt.refcnt() == 1);
                           let mut hdr = pkt.get_mut_header();
                           hdr.swap_addresses();
                           delay_loop(delay);
                       })
        .reset()
        .map(box move |pkt| assert!(pkt.refcnt() == 1))
}
