// https://doc.rust-lang.org/alloc/index.html
#![feature(alloc)]
// Used for cache alignment.
// https://github.com/rust-lang/rust/issues/32838
#![feature(allocator_api)]
#![feature(asm)]
// https://github.com/rust-lang/rust/issues/49733
#![feature(box_syntax)]
// https://github.com/rust-lang/rust/issues/27730
// common workaround: https://github.com/rayon-rs/rayon-hash/blob/master/src/ptr.rs
#![feature(ptr_internals)]
// https://github.com/rust-lang/rust/issues/31844
#![feature(specialization)]

// For cache aware allocation
extern crate alloc;
#[macro_use]
extern crate clap;
extern crate config as config_rs;
#[cfg_attr(test, macro_use)]
extern crate failure;
extern crate fallible_iterator;
extern crate fnv;
#[cfg(any(test, feature = "test"))]
extern crate futures;
extern crate hex;
#[macro_use]
extern crate lazy_static;
extern crate libc;
#[macro_use]
extern crate log;
extern crate net2;
#[cfg(unix)]
extern crate nix;
#[cfg(any(test, feature = "test"))]
#[cfg_attr(any(test, feature = "test"), macro_use)]
extern crate proptest;
extern crate regex;
#[cfg(feature = "sctp")]
extern crate sctp;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate tokio;
extern crate tokio_signal;
extern crate tokio_threadpool;
extern crate twox_hash;

// need these first so other modules in netbricks can use the macros
#[macro_use]
pub mod tests;

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
pub mod runtime;
pub mod scheduler;
pub mod shared_state;
pub mod state;
#[cfg(feature = "test")]
pub mod testing;
pub mod utils;
