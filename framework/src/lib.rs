#![recursion_limit = "1024"]
#![feature(asm)]
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
#![cfg_attr(feature = "dev", deny(warnings))]
extern crate libc;
extern crate byteorder;
extern crate fnv;
extern crate twox_hash;
extern crate regex;
extern crate net2;
#[macro_use]
extern crate lazy_static;
#[cfg(feature = "sctp")]
extern crate sctp;
// TOML for scheduling configuration
extern crate toml;
// UUID for SHM naming
extern crate uuid;

// For cache aware allocation
extern crate alloc;

// Better error handling.
#[macro_use]
extern crate error_chain;

#[cfg(unix)]
extern crate nix;
#[allow(dead_code)]
mod native;
pub mod allocators;
pub mod headers;
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
