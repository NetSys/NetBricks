use e2d2::headers::*;
use e2d2::packet_batch::*;
use std::any::Any;

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

pub fn delay<T: 'static + Batch>(parent: T, delay: u64) -> TransformBatch<MacHeader, ParsedBatch<MacHeader, T>> {
    parent.parse::<MacHeader>()
          .transform(move |hdr: &mut MacHeader, _: &mut [u8], _: Option<&mut Any>| {
              let src = hdr.src.clone();
              hdr.src = hdr.dst;
              hdr.dst = src;
              delay_loop(delay);
          })
}
