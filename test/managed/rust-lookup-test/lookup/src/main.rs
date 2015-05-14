#![feature(box_syntax, libc, core)]
extern crate libc;
extern crate core;
extern crate time;
use time::PreciseTime;
use time::Duration;
use std::ffi::CString;
use std::collections::HashMap;
use std::num::Wrapping;
use std::io::BufReader;
use std::io::BufRead;
use std::fs::File;
use std::path::Path;
extern {
    fn inet_pton(family: i32, 
                 addr: *const libc::c_char,
                 ret: *mut u32) -> u32;
    fn ntohl(addr: u32) -> u32;
}

const TBL_SIZE: usize =  ((1u64 << 24) + 1) as usize;
const TBL_LONG_MASK: u16 = 0x8000;
const LENGTHS:usize = 33;
struct FIB {
    pub tbl24:[u16; TBL_SIZE],
    pub tbl_long:[u16; TBL_SIZE],
}

#[inline]
fn str_to_ipv4(addr: &str) -> u32 {
  let addr_cstring = CString::new(addr.as_bytes()).unwrap();
  unsafe {
    let mut ret = 0u32;
    {
        inet_pton(2, // AF_INET
              addr_cstring.as_ptr(),
              &mut ret);
    }
    return ntohl(ret);
  }
}

fn create_fib(fib: &mut FIB,
              rib: &[HashMap<u32, u16>] ) {
  let mut current_tbl_long:usize = 0; 
  for l in 0..25 {
    for (addr, nhop) in rib[l].iter() {
      let start: usize = (*addr >> 8) as usize;
      let end: usize = start + (1 << (24 - l));
      for pfx in start..end {
        fib.tbl24[pfx] = *nhop;
      }
    }
  }
  for l in 25..LENGTHS {
    for (addr, nhop) in rib[l].iter() {
      let tbl24dest = fib.tbl24[(*addr >> 8) as usize];
      if (tbl24dest & TBL_LONG_MASK) == 0 {
        let start:usize = current_tbl_long + ((*addr & 0xff) as usize);
        let end:usize = start + (1 << (32 - l));
        for pfx in current_tbl_long..(current_tbl_long + 256) {
          if pfx < start || pfx >= end {
            fib.tbl_long[pfx] = tbl24dest;
          } else {
            fib.tbl_long[pfx] = *nhop;
          }
        }
        fib.tbl24[(*addr << 8) as usize] = ((current_tbl_long >> 8) as u16) | TBL_LONG_MASK;
        current_tbl_long += 256;
      } else {
        let long_idx = ((tbl24dest & !(TBL_LONG_MASK)) << 8) as usize;
        let start:usize = long_idx + ((*addr & 0xff) as usize);
        let end:usize = start + (1 << (32 - l));
        for pfx in start..end {
          fib.tbl_long[pfx] = *nhop;
        }
      }
    }
  }
}

#[inline]
fn lookup(fib: &Box<FIB>, 
          ip: u32) -> u16{
  let tbl24_idx: u32 = ip >> 8;
  let tbl24_result = fib.tbl24[tbl24_idx as usize];
  if (tbl24_result & TBL_LONG_MASK) > 0 {
    let idx: u32 = (((tbl24_result & !(TBL_LONG_MASK)) << 8) as u32) + (ip & 0xff);
    return fib.tbl_long[idx as usize];
  } else {
    return tbl24_result;
  }
}


// Do not call from many threads, makes things sad
#[inline]
fn rand_fast() -> u32 {
  static mut seed:Wrapping<u64> = Wrapping(0u64);
  unsafe {
    seed = seed * Wrapping(1103515245u64) +  Wrapping(12345u64);
    return (seed.0 >> 32) as u32;
  }
}

#[inline]
fn benchmark(fib: &Box<FIB>,
             warm: i64,
             batch: u64,
             batches: u32) {
  let mut start = PreciseTime::now();
  while start.to(PreciseTime::now()) < Duration::seconds(warm) {
    lookup(fib, rand_fast());
  }
  let mut done = 0u32;
  let mut lookups = 0u64;
  start = PreciseTime::now();
  while done < batches {
    for _ in 0u64..batch {
      lookup(fib, rand_fast());
      lookups = lookups + 1;
    }
    if start.to(PreciseTime::now()) >= Duration::seconds(1) {
      println!("{} {} {}", start.to(PreciseTime::now()), batch, lookups);
      done = done + 1;
      start = PreciseTime::now();
    }
  }
}

#[inline]
fn new_ip_hash() -> HashMap<u32, u16> {
  return HashMap::new();
}

fn main() {
    //let hash = [HashMap::new(); 33]
    let args:Vec<String> = std::env::args().collect();
    if args.len() < 2 {
      println!("Usage: {} fib", args[0]);
      return;
    }
    let ref fname = args[1];
    println!("Should use {0} as rib", fname);

    let mut hash: Vec<HashMap<u32, u16>> = (0..33).map(|_|{new_ip_hash()}).collect();
    {
        let path = Path::new(fname);
        let file = BufReader::new(File::open(&path).unwrap());
        for line in file.lines() {
          let l = line.unwrap();
          let parts:Vec<&str> = l.split(" ").collect();
          if parts.len() == 2 {
            let addr_parts:Vec<&str> = parts[0].split("/").collect();
            if addr_parts.len() == 2 {
              let len = addr_parts[1].parse::<usize>().unwrap();
              let addr = str_to_ipv4(addr_parts[0]);
              let nhop = parts[1].parse::<u16>().unwrap();
              hash[len].insert(addr, nhop);
              println!("Inserting {}({}) {} {}", addr_parts[0], addr, len, nhop);
            }
          }
        }
    }
    //let fib = Box<FIB>
    println!("Creating FIB");
    let mut fib = box FIB{tbl24: [0u16;TBL_SIZE], tbl_long: [0u16;TBL_SIZE]};
    {
        create_fib(&mut fib, &hash[..]);
    }
    println!("Done creating FIB");
    const WARM:i64 = 1;
    const BATCH_SIZE:u64 = 10;
    const BATCHES:u32 = 5;
    for exp in 0..BATCH_SIZE {
      benchmark(&fib, WARM, (1u64 << exp), BATCHES);
    }
}
