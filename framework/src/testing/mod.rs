//! Utilities for writing tests
//!
//! To compile the utilities with NetBricks, enable the feature `test`
//! in the project's Cargo.toml.
//!
//! # Example
//!
//! ```
//! [dev-dependencies]
//! netbricks = { version = "1.0", features = ["test"] }
//! ```

mod arbitrary;
mod packet;
mod strategy;

pub use self::packet::*;
pub use self::strategy::*;
pub use netbricks_codegen::dpdk_test;
pub use tokio::prelude::future::{lazy, Future};

lazy_static! {
    pub static ref DPDK_TEST_POOL: tokio_threadpool::ThreadPool = tokio_threadpool::Builder::new()
        .pool_size(1)
        .after_start(|| crate::interface::dpdk::init_system_wl("dpdk_tests", 0, &[]))
        .build();
}
