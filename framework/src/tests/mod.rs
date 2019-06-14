pub use futures::future::lazy;

lazy_static! {
    pub static ref DPDK_POOL: tokio_threadpool::ThreadPool = tokio_threadpool::Builder::new()
        .panic_handler(|err| std::panic::resume_unwind(err))
        .pool_size(1)
        .after_start(|| { ::interface::dpdk::init_system_wl("dpdk_tests", 0, &[]) })
        .build();
}

#[macro_export]
macro_rules! dpdk_test {
    ($($arg:tt)*) => {
        $crate::tests::DPDK_POOL.spawn($crate::tests::lazy(|| {
            { $($arg)* };
            Ok(())
        }));
    }
}
