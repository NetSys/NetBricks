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
    ($test: block) => {
        let result = DPDK_THREAD.install(
            || {
                panic::catch_unwind(|| {
                    $test
                })
            }
        );

        if let Err(err) = result {
            panic::resume_unwind(err);
        }
    }
}
