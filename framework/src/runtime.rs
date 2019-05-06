use allocators::CacheAligned;
use common::Result;
use config::{NetBricksConfiguration, CLI_ARGS};
use interface::PortQueue;
use scheduler::{initialize_system, NetBricksContext, StandaloneScheduler};
use std::io::{Error, ErrorKind};
use std::sync::mpsc::{sync_channel, TryRecvError};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::prelude::future::{poll_fn, Either};
use tokio::prelude::{Async, Future, Stream};
use tokio::timer::{Delay, Interval};
use tokio_signal::unix::Signal;

pub use tokio_signal::unix::{SIGHUP, SIGINT, SIGTERM};

type TokioRuntime = tokio::runtime::current_thread::Runtime;

pub struct Runtime {
    context: NetBricksContext,
    tokio_rt: TokioRuntime,
    on_signal: Arc<Fn(i32) -> std::result::Result<(), i32>>,
}

impl Runtime {
    /// Intializes the NetBricks context and starts the background schedulers
    pub fn init(configuration: &NetBricksConfiguration) -> Result<Runtime> {
        info!("initializing context:\n{}", configuration);
        let mut context = initialize_system(configuration)?;
        context.start_schedulers();
        let tokio_rt = TokioRuntime::new()?;
        Ok(Runtime {
            context,
            tokio_rt,
            on_signal: Arc::new(|_| Err(0)),
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
    ///
    /// # Example
    ///
    /// ```
    /// let mut runtime = Runtime::init(&CONFIG)?;
    /// runtime.set_on_signal(|signal| match signal {
    ///     SIGHUP => {
    ///         if let Ok(config) = reload() {
    ///             CONFIG.set(config);
    ///             Ok(())
    ///         } else {
    ///             Err(1)
    ///         }
    ///     }
    ///     _ => Err(0),
    /// });
    /// runtime.execute()
    /// ```
    pub fn set_on_signal<T>(&mut self, on_signal: T)
    where
        T: Fn(i32) -> std::result::Result<(), i32> + 'static,
    {
        self.on_signal = Arc::new(on_signal);
    }

    /// Adds a repeated task to run at an interval
    pub fn add_task_to_run<T>(&mut self, task: T, interval: Duration)
    where
        T: Fn() -> () + 'static,
    {
        let task = Interval::new_interval(interval)
            .for_each(move |_| Ok(task()))
            .map_err(|e| warn_chain!(&e.into()));
        self.tokio_rt.spawn(task);
    }

    fn shutdown(&mut self) {
        info!("shutting down context");
        self.context.shutdown();
    }

    fn wait_for_timeout(&mut self) -> Result<()> {
        let duration = value_t!(CLI_ARGS, "duration", u64)?;
        let when = Instant::now() + Duration::from_secs(duration);

        info!("waiting for {} seconds", duration);
        let main_loop = Delay::new(when);
        let res = self.tokio_rt.block_on(main_loop);

        self.shutdown();
        res.map_err(|e| e.into())
    }

    fn wait_for_unix_signal(&mut self) -> Result<()> {
        let sighup = Signal::new(SIGHUP).flatten_stream();
        let sigint = Signal::new(SIGINT).flatten_stream();
        let sigterm = Signal::new(SIGTERM).flatten_stream();
        let stream = sighup.select(sigint).select(sigterm);

        let (sender, receiver) = sync_channel(1);
        let on_signal = self.on_signal.clone();

        // listens for Unix signals. when run, this future will block and never
        // resolve. when an exit code is returned by the signal handler, it will
        // send a message to a second future to resolve and unblock the thread.
        let main_loop = stream.for_each(move |signal| {
            if let Err(code) = on_signal(signal) {
                sender
                    .try_send(code)
                    .map_err(|e| Error::new(ErrorKind::BrokenPipe, e))
            } else {
                Ok(())
            }
        });

        // the second future waits for a message to resolve and unblock.
        let main_loop = main_loop.select2(poll_fn(move || match receiver.try_recv() {
            Ok(code) => Ok(Async::Ready(code)),
            Err(TryRecvError::Empty) => Ok(Async::NotReady),
            Err(e) => Err(e),
        }));

        match self.tokio_rt.block_on(main_loop) {
            Ok(Either::A(_)) => unreachable!(),
            Ok(Either::B((code, _))) => {
                self.shutdown();
                info!("exiting with code {}", code);
                std::process::exit(code);
            }
            Err(Either::A((e, _))) => {
                self.shutdown();
                Err(e.into())
            }
            Err(Either::B((e, _))) => {
                self.shutdown();
                Err(e.into())
            }
        }
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
