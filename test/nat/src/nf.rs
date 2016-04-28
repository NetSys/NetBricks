use e2d2::headers::*;
use e2d2::packet_batch::*;
use e2d2::utils::*;
use e2d2::scheduler::*;

#[derive(Clone, Default)]
struct Unit;

pub fn nat<T: 'static + Batch>(parent: T, s: &mut Scheduler) -> CompositionBatch {
    let mut groups = parent.parse::<MacHeader>()
        .context::<Flow>()
        .transform(box move |hdr, payload, ctx| {
            // Let us first reverse MAC.
            let src = hdr.src.clone();
            hdr.src = hdr.dst;
            hdr.dst = src;
            // Extracted flow
            if let Some(flow) = ipv4_extract_flow(hdr, payload) {
                match ctx.and_then(|c| c.downcast_mut::<Flow>()) {
                    Some(f) => *f = flow,
                    None => panic!("Could not find context")
                };
            }
        })
        .group_by::<Flow>(2, box move |_, _, ctx| {
            if let Some(flow) = ctx.and_then(|c| c.downcast_mut::<Flow>()) {
                if flow.proto == 0x06 || flow.proto == 0x11 {
                    //let box_flow = box flow.clone();
                    //(1, Some(box_flow))
                    (1, None)
                } else {
                    (0, None)
                }
            } else {
                (0, None)
            }
        }, s);
        let g0 = groups.get_group(0).unwrap().compose();
        let g1 = groups.get_group(1).unwrap()
                       .parse::<MacHeader>()
                       .map(box move |hdr, payload, ctx| {
                           //if let Some(curr_flow) = ipv4_extract_flow(hdr, payload) {
                           //} else {
                               //panic!("Could not extract flow");
                           //}
                           if let Some(flow) = ctx.and_then(|c| c.downcast_mut::<Box<Flow>>()) {
                           } else {
                               panic!("Could not find flow info");
                           }
                        }).compose();
        merge(vec![g0, g1]).compose()
}
