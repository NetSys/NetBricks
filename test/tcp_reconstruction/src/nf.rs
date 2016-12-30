use e2d2::headers::*;
use e2d2::operators::*;
use e2d2::scheduler::*;
use e2d2::state::*;
use e2d2::utils::Flow;
use fnv::FnvHasher;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::hash::BuildHasherDefault;
use std::str;

type FnvHash = BuildHasherDefault<FnvHasher>;
const BUFFER_SIZE: usize = 2048;
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
                  box move |p| { if p.get_header().protocol() == 6 { 0 } else { 1 } },
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
                let seq = p.get_header().seq_num();
                match cache.entry(*flow) {
                    Entry::Occupied(mut e) => {
                        let reset = p.get_header().rst_flag();
                        {
                            let entry = e.get_mut();
                            let result = entry.add_data(seq, p.get_payload());
                            match result {
                                InsertionResult::Inserted { available, .. } => {
                                    if available > PRINT_SIZE {
                                        let mut read = 0;
                                        while available - read > PRINT_SIZE {
                                            let avail = entry.read_data(&mut read_buf[..]);
                                            read += avail;
                                        }
                                    }
                                }
                                InsertionResult::OutOfMemory { written, .. } => {
                                    if written == 0 {
                                        // println!("Resetting since receiving data that is too far ahead");
                                        entry.reset();
                                        entry.seq(seq, p.get_payload());
                                    }
                                }
                            }
                        }
                        if reset {
                            // Reset handling.
                            e.remove_entry();
                        }
                    }
                    Entry::Vacant(e) => {
                        match ReorderedBuffer::new(BUFFER_SIZE) {
                            Ok(mut b) => {
                                if !p.get_header().syn_flag() {
                                }
                                let result = b.seq(seq, p.get_payload());
                                match result {
                                    InsertionResult::Inserted { available, .. } => {
                                        if available > PRINT_SIZE {
                                            let mut read = 0;
                                            while available - read > PRINT_SIZE {
                                                let avail = b.read_data(&mut read_buf[..]);
                                                read += avail;
                                            }
                                        }
                                    }
                                    InsertionResult::OutOfMemory { .. } => {
                                        println!("Too big a packet?");
                                    }
                                }
                                e.insert(b);
                            }
                            Err(_) => (),
                        }
                    }
                }
            }
        })
        .compose();
    merge(vec![pipe, groups.get_group(1).unwrap().compose()]).compose()
}
