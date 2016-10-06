use e2d2::utils::Flow;
use e2d2::headers::*;
use e2d2::scheduler::*;
use e2d2::operators::*;
use e2d2::state::*;
use std::str;
use std::collections::HashMap;
use fnv::FnvHasher;
use std::hash::BuildHasherDefault;

type FnvHash = BuildHasherDefault<FnvHasher>;
const BUFFER_SIZE: usize = 10240; // 20K of buffers
const PRINT_SIZE: usize = 256;

pub fn reconstruction<T: 'static + Batch<Header = NullHeader>>(parent: T, sched: &mut Scheduler) -> CompositionBatch {
    let mut cache = HashMap::<Flow, ReorderedBuffer, FnvHash>::with_hasher(Default::default());
    let mut read_buf: Vec<u8> = (0..PRINT_SIZE).map(|_| 0).collect();
    let mut groups = parent.parse::<MacHeader>()
        .transform(box move |p| {
            p.get_mut_header().swap_addresses();
        })
        .parse::<IpHeader>()
        .group_by(2,
                  box move |p| {
            if p.get_header().protocol() == 6 {
                0
            } else {
                1
            }
        },
                  sched);

    let pipe = groups.get_group(0)
        .unwrap()
        .metadata(box move |p| {
            let flow = p.get_header().flow().unwrap();
            flow
        })
        .parse::<TcpHeader>()
        .transform(box move |p| {
            if !p.get_header().psh_flag() {
                let flow = p.read_metadata();
                let mut e = cache.entry(*flow).or_insert_with(|| ReorderedBuffer::new(BUFFER_SIZE));
                let seq = p.get_header().seq_num();
                let result = if p.get_header().syn_flag() {
                    e.seq(seq, p.get_payload())
                } else if e.is_established() {
                    e.add_data(seq, p.get_payload())
                } else {
                    e.seq(seq, p.get_payload())
                };
                match result {
                    InsertionResult::Inserted { available, .. } => {
                        if available > PRINT_SIZE {
                            let mut read = 0;
                            while available - read > PRINT_SIZE {
                                let avail = e.read_data(&mut read_buf[0..]);
                                read += avail;
                                match str::from_utf8(&read_buf[0..avail]) {
                                    Ok(_s) => {
                                        // println!("Read {}", s);
                                    }
                                    _ => (),
                                }
                                println!("Read {}", str::from_utf8(&read_buf[0..avail]).unwrap());
                            }
                        }
                    }
                    InsertionResult::OutOfMemory { .. } => {
                        // InsertionResult::OutOfMemory { written, available } => {
                        // println!("{}", p.get_header());
                    }
                };
                if p.get_header().rst_flag() {
                    e.reset();
                }
            }
        })
        .compose();
    merge(vec![pipe, groups.get_group(1).unwrap().compose()]).compose()
}
