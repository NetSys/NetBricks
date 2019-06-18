#![feature(proc_macro_span, proc_macro_diagnostic)]
#![recursion_limit = "128"]

extern crate proc_macro;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// Procedural Macro for running dpdk-tests within a single-thread.
///
/// # Example
///
/// ```
/// #[cfg(test)]
/// pub mod tests {
///     use super::*;
///     use netbricks::testing::dpdk_test;
///
///     #[dpdk_test]
///     fn test_drop() {
///         ...
///         assert!(drop);
///     }
/// ```
#[proc_macro_attribute]
pub fn dpdk_test(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);

    let test = &input.ident;
    let block = &input.block;

    let run = quote! {
        #[test]
        fn #test() {
            let f = ::netbricks::testing::DPDK_TEST_POOL.spawn_handle(::netbricks::testing::lazy(|| {
                ::std::panic::catch_unwind(|| {
                    #block
                })
            }));

            if let Err(e) = ::netbricks::testing::Future::wait(f) {
                ::std::panic::resume_unwind(e);
            }
        }
    };

    run.into()
}
