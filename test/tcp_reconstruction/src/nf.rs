use e2d2::utils::Flow;
use e2d2::headers::*;
use e2d2::operators::*;
use std::collections::HashMap;
use fnv::FnvHasher;
use std::hash::BuildHasherDefault;

type FnvHash = BuildHasherDefault<FnvHasher>;

pub fn reconstruction<T: 'static + Batch<Header = NullHeader>>(parent: T) -> CompositionBatch {
    let mut cache = HashMap::<Flow, usize, FnvHash>::with_hasher(Default::default());
    parent.parse::<MacHeader>()
        .transform(box move |p| {
            p.get_mut_header().swap_addresses();
        })
        .parse::<IpHeader>()
        .filter(box move |p| p.get_header().protocol() == 6)
        .metadata(box move |p| {
            let flow = p.get_header().flow().unwrap();
            flow
        })
        .parse::<TcpHeader>()
        .transform(box move |p| {
            let flow = p.read_metadata();
            let mut e = cache.entry(*flow).or_insert_with(|| 0);
            *e = *e + 1;
        })
        .compose()
}
