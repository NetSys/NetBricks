#![feature(asm)]
#![feature(repr_simd)]
#![feature(log_syntax)]
#![feature(box_syntax)]
#![feature(specialization)]
#![feature(associated_consts)]
#![feature(slice_concat_ext)]
#![feature(fnbox)]
#![feature(alloc)]
#![feature(heap_api)]
#![feature(unique)]

#![allow(unused_features)]
#![feature(integer_atomics)]

#![cfg_attr(feature = "dev", allow(unstable_features))]
// Need this since PMD port construction triggers too many arguments.
#![cfg_attr(feature = "dev", allow(too_many_arguments))]
#![cfg_attr(feature = "dev", feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]
extern crate libc;
extern crate byteorder;
extern crate fnv;
extern crate twox_hash;
extern crate regex;
extern crate net2;
#[macro_use]
extern crate lazy_static;
#[cfg(feature="sctp")]
extern crate sctp;
/// TOML for scheduling configuration
extern crate toml;
/// UUID for SHM naming
extern crate uuid;

/// For cache aware allocation
extern crate alloc;

#[cfg(unix)]
extern crate nix;
pub mod allocators;
pub mod headers;
mod io;
pub mod scheduler;
pub mod utils;
pub mod queues;
pub mod state;
pub mod operators;
pub mod interface;
pub mod common;
pub mod control;
pub mod shared_state;
pub mod config;
