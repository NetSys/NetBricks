#![recursion_limit = "1024"]
#![feature(asm)]
#![feature(log_syntax)]
#![feature(box_syntax)]
#![feature(specialization)]
#![feature(slice_concat_ext)]
#![feature(fnbox)]
#![feature(alloc)]
#![feature(heap_api)]
#![feature(unique)]
#![feature(const_fn)]
#![feature(ip_constructors)]
#![feature(type_ascription)]
// FIXME: Figure out if this is really the right thing here.
#![feature(ptr_internals)]
#![feature(iterator_step_by)]
#![allow(safe_packed_borrows)]
// Used for cache alignment.
#![feature(allocator_api)]
#![allow(unused_features)]
#![allow(renamed_and_removed_lints)]
#![feature(integer_atomics)]
#![allow(unused_doc_comments)]
#![cfg_attr(feature = "dev", allow(unstable_features))]
// Need this since PMD port construction triggers too many arguments.
#![cfg_attr(feature = "dev", allow(too_many_arguments))]
#![cfg_attr(feature = "dev", feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]
#![cfg_attr(feature = "dev", deny(warnings))]
extern crate byteorder;
extern crate fnv;
#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate net2;
extern crate num;

#[macro_use]
extern crate num_derive;

extern crate regex;
#[cfg(feature = "sctp")]
extern crate sctp;
extern crate twox_hash;
// TOML for scheduling configuration
extern crate toml;
#[macro_use]
extern crate serde_derive;
// UUID for SHM naming
extern crate uuid;

// For cache aware allocation
extern crate alloc;

// Handle generic, compile-time arrays in structs
extern crate generic_array;

// Better error handling.
#[macro_use]
extern crate error_chain;

// Bring in crossbeam synchronization primatives
extern crate crossbeam;

#[cfg(unix)]
extern crate nix;
pub mod allocators;
pub mod common;
pub mod config;
pub mod control;
pub mod headers;
pub mod interface;
#[allow(dead_code)]
mod native;
mod native_include;
pub mod operators;
pub mod queues;
pub mod scheduler;
pub mod shared_state;
pub mod state;
pub mod utils;
