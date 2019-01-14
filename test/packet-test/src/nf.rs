use netbricks::headers::*;
use netbricks::operators::*;
pub fn delay<T: 'static + Batch<Header = NullHeader>>(parent: T) -> TransformBatch<NullHeader, T> {
    let mut m = MacHeader::new();
    let dst = MacAddress {
        addr: [0x68, 0x05, 0xca, 0x33, 0xff, 0x79],
    };
    let src = MacAddress {
        addr: [0x68, 0x05, 0xca, 0x33, 0xfd, 0xc8],
    };
    m.set_dst(dst);
    m.set_src(src);
    m.set_etype(mac::EtherType::IPv4);
    parent.transform(box move |pkt| {
        pkt.write_header(&m, 0).unwrap();
    })
    // parent.parse::<MacHeader>()
    // .transform(box move |pkt| {
    // assert!(pkt.refcnt() == 1);
    // let mut hdr = pkt.get_mut_header();
    // /let src = hdr.src;
    // hdr.src[2] += 1;
    // hdr.dst[1] += 1;
    // delay_loop(delay);
    // })
}
