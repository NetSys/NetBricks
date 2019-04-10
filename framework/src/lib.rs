#![recursion_limit = "1024"]
#![feature(asm)]
#![feature(box_syntax)]
#![feature(specialization)]
#![feature(fnbox)]
#![feature(alloc)]
#![feature(result_map_or_else)]
#![feature(const_fn)]
#![feature(custom_attribute)]
#![feature(type_ascription)]
// FIXME: Figure out if this is really the right thing here.
#![feature(ptr_internals)]
#![allow(safe_packed_borrows)]
// Used for cache alignment.
#![feature(allocator_api)]
#![allow(unused_features)]
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
extern crate hex;
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

extern crate serde;
#[macro_use]
extern crate serde_derive;
// UUID for SHM naming
extern crate uuid;

// For cache aware allocation
extern crate alloc;

// Handle generic, compile-time arrays in structs
extern crate generic_array;

// Error Handling
#[cfg_attr(test, macro_use)]
extern crate failure;

// Bring in crossbeam synchronization primatives
extern crate crossbeam;

// Handle execution on other threads
extern crate rayon;

extern crate getopts;

#[macro_use]
extern crate log;

#[cfg(unix)]
extern crate nix;

extern crate fallible_iterator;

// need these first so other modules in netbricks can use the macros
#[macro_use]
pub mod common;
pub mod tests;

pub mod allocators;
pub mod config;
pub mod control;
pub mod headers;
pub mod interface;
#[allow(dead_code)]
mod native;
mod native_include;
pub mod new_operators;
pub mod operators;
pub mod packets;
pub mod queues;
pub mod scheduler;
pub mod shared_state;
pub mod state;
pub mod utils;
