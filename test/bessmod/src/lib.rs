#![feature(box_syntax)]
#![feature(asm)]

extern crate e2d2;
extern crate fnv;
extern crate getopts;
extern crate rand;

pub mod bessnb;
mod nf;

pub use bessnb::init_mod;
pub use bessnb::deinit_mod;
pub use bessnb::run_once;
pub use bessnb::get_stats;
