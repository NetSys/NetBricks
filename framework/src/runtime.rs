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
    on_signal: Box<Fn(i32) -> std::result::Result<(), i32>>,
}

impl Runtime {
    /// Intializes the NetBricks context and starts the background schedulers
    pub fn init(configuration: &NetBricksConfiguration) -> Result<Runtime> {
        info!("initializing context:\n{}", configuration);
        let mut context = initialize_system(configuration)?;
        context.start_schedulers();
        Ok(Runtime {
            context,
            on_signal: Box::new(|_| Err(0)),
        })
    }

    /// Runs a packet processing pipeline installer
    pub fn add_pipeline_to_run<T>(&mut self, installer: T)
    where
        T: Fn(Vec<CacheAligned<PortQueue>>, &mut StandaloneScheduler) + Send + Sync + 'static,
    {
        self.context.add_pipeline_to_run(Arc::new(installer));
    }

    /// Sets the Unix signal handler
    ///
    /// `SIGHUP`, `SIGINT` and `SIGTERM` are the supported Unix signals.
    /// The return of the handler determines whether to terminate the
    /// process. `Ok(())` indicates to keep the process running, and
    /// `Err(i32)` indicates to exit the process with the given exit
    /// code. If no signal handler is provided, the process will terminate
    /// on any signal received.
    pub fn set_on_signal<T>(&mut self, on_signal: T)
    where
        T: Fn(i32) -> std::result::Result<(), i32> + 'static,
    {
        self.on_signal = Box::new(on_signal);
    }

    fn wait_for_timeout(&mut self) -> Result<()> {
        let duration = value_t!(CLI_ARGS, "duration", u64)?;
        let when = Instant::now() + Duration::from_secs(duration);

        let main_loop = Delay::new(when).and_then(|_| {
            info!("shutting down context");
            self.context.shutdown();
            Ok(())
        });

        info!("waiting for {} seconds", duration);
        current_thread::block_on_all(main_loop).map_err(|e| e.into())
    }

    fn wait_for_unix_signal(&mut self) -> Result<()> {
        let sighup = Signal::new(SIGHUP).flatten_stream();
        let sigint = Signal::new(SIGINT).flatten_stream();
        let sigterm = Signal::new(SIGTERM).flatten_stream();
        let stream = sighup.select(sigint).select(sigterm);

        let main_loop = stream.for_each(|signal| {
            if let Err(code) = (self.on_signal)(signal) {
                info!("shutting down context");
                self.context.shutdown();
                info!("exiting with code {}", code);
                std::process::exit(code);
            }

            Ok(())
        });

        current_thread::block_on_all(main_loop).map_err(|e| e.into())
    }

    /// Executes tasks and pipelines
    ///
    /// If a timeout is provided through command line argument `--duration`,
    /// the runtime will wait for specified value in seconds and then terminate
    /// the process. Otherwise, it will wait for a Unix signal before exiting.
    /// By default, any Unix signal received will end the process. To change
    /// this behavior, use `set_on_signal` to customize signal handling.
    pub fn execute(&mut self) -> Result<()> {
        self.context.execute();

        if CLI_ARGS.is_present("duration") {
            self.wait_for_timeout()
        } else {
            self.wait_for_unix_signal()
        }
    }
}
