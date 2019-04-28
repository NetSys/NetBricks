use allocators::CacheAligned;
use common::Result;
use config::{NetBricksConfiguration, CLI_ARGS};
use interface::PortQueue;
use scheduler::{initialize_system, NetBricksContext, StandaloneScheduler};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::prelude::Future;
use tokio::runtime::current_thread;
use tokio::timer::Delay;

pub struct Runtime {
    context: NetBricksContext,
}

impl Runtime {
    /// Intializes the NetBricks context and starts the background schedulers
    pub fn init(configuration: &NetBricksConfiguration) -> Result<Runtime> {
        info!("initializing NetBricks context:\n{}", configuration);
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

    /// Executes pipelines in test mode
    ///
    /// Runtime will wait for a delay before exiting in test mode. The
    /// delay is specified through the command line `--duration n`.
    pub fn execute_test(&mut self) -> Result<()> {
        let duration = value_t!(CLI_ARGS, "duration", u64)?;

        self.context.execute();

        let when = Instant::now() + Duration::from_secs(duration);
        let shutdown = Delay::new(when).and_then(|_| {
            info!("shutting down NetBricks context");
            self.context.shutdown();
            Ok(())
        });

        info!("waiting for {} seconds", duration);
        current_thread::block_on_all(shutdown).map_err(|e| e.into())
    }
}
