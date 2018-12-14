#![feature(box_syntax)]
#![feature(asm)]
#[macro_use]
extern crate log;
extern crate simplelog;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
extern crate config;
extern crate futures;
extern crate netbricks;
extern crate tokio;
extern crate tokio_signal;
use self::nf::*;
use config::{Config, ConfigError, File, FileFormat};
use log::Level;
use netbricks::config::{basic_opts, read_matches};
use netbricks::interface::*;
use netbricks::operators::*;
use netbricks::scheduler::*;
use netbricks::utils::Atom;
use simplelog::{Config as SimpleConfig, LevelFilter, WriteLogger};
use std::env;
use std::fmt::Display;
use std::fs::File as StdFile;
use std::process;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::prelude::{Future, Stream};
use tokio_signal::unix::{Signal, SIGHUP, SIGUSR1};
mod nf;

#[derive(Debug, Deserialize)]
pub struct Foo {
    pub baz: bool,
    pub bar: Bar,
}

#[derive(Debug, Deserialize)]
pub struct Bar {
    pub baz: bool,
}

static DEFAULT_TOML: &'static str = r#"
    baz = true
    [bar]
      baz = false
"#;

// create a static atom-like wrapper for the non-dpdk configuration information
lazy_static! {
    static ref ATOM_CONF: Atom<Bar> = {
        if let Ok(conf) = read_app_configuration() {
            Atom::new(conf.bar)
        } else {
            process::exit(1);
        }
    };
}

pub fn read_app_configuration() -> Result<Foo, ConfigError> {
    let mut config = Config::new();
    config.merge(File::from_str(DEFAULT_TOML, FileFormat::Toml))?;
    config.try_into()
}

fn handle_signals(configuration: &'static Atom<Bar>) {
    let sighup = Signal::new(SIGHUP).flatten_stream();
    let sigusr1 = Signal::new(SIGUSR1).flatten_stream();

    let on_signal = sighup.select(sigusr1).for_each(move |signal| {
        match signal {
            SIGHUP | SIGUSR1 => {
                if let Ok(new_config) = read_app_configuration() {
                    let mut nf_stuff = new_config.bar;
                    nf_stuff.baz = true;
                    configuration.set(nf_stuff);
                    info!(
                        "So long, and thanks for all the fish: {:?}",
                        configuration.get()
                    );
                }
            }
            _ => println!("Unhandled UNIX Signal."),
        }
        Ok(())
    });

    thread::spawn(move || tokio::run(on_signal.map_err(|err| panic!("{}", err))));
}

fn start_logger() {
    WriteLogger::init(
        LevelFilter::Info,
        SimpleConfig {
            time: None,
            level: Some(Level::Error),
            target: Some(Level::Debug),
            location: Some(Level::Trace),
            time_format: None,
        },
        StdFile::create("test.log").unwrap(),
    )
    .unwrap();
}

fn test<T, S>(ports: Vec<T>, sched: &mut S)
where
    T: PacketRx + PacketTx + Display + Clone + 'static,
    S: Scheduler + Sized,
{
    println!("Receiving started");

    let pipelines: Vec<_> = ports
        .iter()
        .map(|port| nf(ReceiveBatch::new(port.clone()), sched).send(port.clone()))
        .collect();
    println!("Running {} pipelines", pipelines.len());
    for pipeline in pipelines {
        sched.add_task(pipeline).unwrap();
    }

    handle_signals(&ATOM_CONF);
}

fn main() {
    start_logger();

    let mut opts = basic_opts();
    opts.optopt(
        "",
        "dur",
        "Test duration",
        "If this option is set to a nonzero value, then the \
         test will just loop after 2 seconds",
    );

    let args: Vec<String> = env::args().collect();
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };

    let configuration = read_matches(&matches, &opts);

    let test_duration: u64 = matches
        .opt_str("dur")
        .unwrap_or_else(|| String::from("0"))
        .parse()
        .expect("Could not parse test duration");

    match initialize_system(&configuration) {
        Ok(mut context) => {
            context.start_schedulers();
            context.add_pipeline_to_run(Arc::new(move |p, s: &mut StandaloneScheduler| test(p, s)));
            context.execute();

            if test_duration != 0 {
                thread::sleep(Duration::from_secs(test_duration));
            } else {
                loop {
                    thread::sleep(Duration::from_secs(2));
                }
            }
        }
        Err(ref e) => {
            println!("Error: {}", e);
            if let Some(backtrace) = e.backtrace() {
                println!("Backtrace: {:?}", backtrace);
            }
            process::exit(1);
        }
    }
}
