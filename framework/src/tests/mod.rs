pub use tokio::prelude::future::{lazy, Future};

lazy_static! {
    pub static ref DPDK_POOL: tokio_threadpool::ThreadPool = tokio_threadpool::Builder::new()
        .pool_size(1)
        .after_start(|| crate::interface::dpdk::init_system_wl("dpdk_tests", 0, &[]))
        .build();
}

#[macro_export]
macro_rules! dpdk_test {
    ($($arg:tt)*) => {
        let f = $crate::tests::DPDK_POOL.spawn_handle(
            $crate::tests::lazy(|| {
                std::panic::catch_unwind(|| {
                    $($arg)*
                })
            })
        );
        if let Err(e) = $crate::tests::Future::wait(f) {
            std::panic::resume_unwind(e);
        }
    }
}
