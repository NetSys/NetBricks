//#![cfg(test)]
#![macro_use]

use interface::{dpdk};
use rayon::{ThreadPool, ThreadPoolBuilder};

lazy_static! {
    pub static ref DPDK_THREAD: ThreadPool = {
        ThreadPoolBuilder::new()
            .num_threads(1)
            .start_handler(|_index| {
                dpdk::init_system_wl("dpdk_tests", 0, &[]);
            })
            .build()
            .unwrap()
    };
}

#[macro_export]
macro_rules! dpdk_test {
    ($($arg:tt)*) => {
        use ::std::panic::{catch_unwind, resume_unwind};

        let result = $crate::tests::DPDK_THREAD.install(
            || {
                catch_unwind(|| {
                    $($arg)*
                })
            }
        );

        if let Err(err) = result {
            resume_unwind(err);
        }
    }
}
