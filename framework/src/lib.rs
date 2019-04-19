#![recursion_limit = "1024"]
#![feature(asm)]
#![feature(box_syntax)]
#![feature(alloc)]
#![feature(const_fn)]
#![feature(specialization)]
#![feature(type_ascription)]
// FIXME: Figure out if this is really the right thing here.
#![feature(ptr_internals)]
#![allow(safe_packed_borrows)]
// Used for cache alignment.
#![feature(allocator_api)]
extern crate byteorder;
extern crate fnv;
#[macro_use]
extern crate lazy_static;
extern crate hex;
extern crate libc;
extern crate net2;
extern crate regex;
#[cfg(feature = "sctp")]
extern crate sctp;
extern crate twox_hash;
// TOML for scheduling configuration
extern crate toml;

extern crate serde;
#[macro_use]
extern crate serde_derive;

// For cache aware allocation
extern crate alloc;

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

#[cfg(test)]
#[macro_use]
extern crate proptest;

// need these first so other modules in netbricks can use the macros
#[macro_use]
pub mod common;
pub mod allocators;
pub mod config;
pub mod control;
pub mod interface;
#[allow(dead_code)]
mod native;
mod native_include;
pub mod operators;
pub mod packets;
pub mod scheduler;
pub mod shared_state;
pub mod state;
pub mod tests;
pub mod utils;
