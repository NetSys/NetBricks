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

pub(crate) mod arbitrary;
pub(crate) mod packet;
pub(crate) mod strategy;

pub use self::packet::*;
pub use self::strategy::*;
