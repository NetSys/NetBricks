#![feature(repr_simd)]
#![feature(log_syntax)]
#![feature(box_syntax)]
#![feature(type_macros)]
#![cfg_attr(feature = "dev", allow(unstable_features))]
// We need this since rx_cores and tx_cores triggers a similar names warning.
#![cfg_attr(feature = "dev", allow(similar_names))]
// Need this since PMD port construction triggers too many arguments.
#![cfg_attr(feature = "dev", allow(too_many_arguments))]
#![cfg_attr(feature = "dev", feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]
extern crate libc;
extern crate byteorder;
pub mod headers;
pub mod io;
pub mod utils;
