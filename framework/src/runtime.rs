use allocators::CacheAligned;
use common::Result;
use config::{NetBricksConfiguration, CLI_ARGS};
use interface::PortQueue;
use scheduler::{initialize_system, NetBricksContext, StandaloneScheduler};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::prelude::{Future, Stream};
use tokio::runtime::current_thread;
use tokio::timer::Delay;
use tokio_signal::unix::Signal;

pub use tokio_signal::unix::{SIGHUP, SIGINT, SIGTERM};

pub struct Runtime {
    context: NetBricksContext,
}

impl Runtime {
    /// Intializes the NetBricks context and starts the background schedulers
    pub fn init(configuration: &NetBricksConfiguration) -> Result<Runtime> {
        info!("initializing context:\n{}", configuration);
        let mut context = initialize_system(configuration)?;
        context.start_schedulers();
        Ok(Runtime { context })
    }

    /// Runs a packet processing pipeline installer
    pub fn add_pipeline_to_run<T>(&mut self, installer: T)
    where
        T: Fn(Vec<CacheAligned<PortQueue>>, &mut StandaloneScheduler) + Send + Sync + 'static,
    {
        self.context.add_pipeline_to_run(Arc::new(installer));
    }

    /// Executes tasks and pipelines
    pub fn execute<T>(&mut self, on_signal: T) -> Result<()>
    where
        T: Fn(i32) -> std::result::Result<(), i32>,
    {
        let sighup = Signal::new(SIGHUP).flatten_stream();
        let sigint = Signal::new(SIGINT).flatten_stream();
        let sigterm = Signal::new(SIGTERM).flatten_stream();
        let stream = sighup.select(sigint).select(sigterm);

        let main_loop = stream.for_each(|signal| {
            let code = match on_signal(signal) {
                Ok(()) => 0,
                Err(err) => err,
            };

            if signal != SIGHUP || code != 0 {
                info!("shutting down context");
                self.context.shutdown();
                info!("exiting with code {}", code);
                std::process::exit(code);
            }

            Ok(())
        });

        current_thread::block_on_all(main_loop).map_err(|e| e.into())
    }

    /// Executes tasks and pipelines in test mode
    ///
    /// Runtime will wait for a delay before exiting in test mode. The
    /// delay is specified through the command line `--duration n`.
    pub fn execute_test(&mut self) -> Result<()> {
        let duration = value_t!(CLI_ARGS, "duration", u64)?;

        self.context.execute();

        let when = Instant::now() + Duration::from_secs(duration);
        let main_loop = Delay::new(when).and_then(|_| {
            info!("shutting down context");
            self.context.shutdown();
            Ok(())
        });

        info!("waiting for {} seconds", duration);
        current_thread::block_on_all(main_loop).map_err(|e| e.into())
    }
}
