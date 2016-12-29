use common::*;
use std::thread::{self, JoinHandle};
use std::sync::mpsc::{SyncSender, sync_channel};
use interface::{PmdPort, PortQueue};
use interface::dpdk::{init_system, init_thread};
use allocators::CacheAligned;
use std::sync::Arc;
use std::collections::HashMap;
use std::collections::HashSet;
use scheduler::*;
use config::NetbricksConfiguration;

type AlignedPortQueue = CacheAligned<PortQueue>;
#[derive(Default)]
pub struct NetBricksContext {
    pub ports: HashMap<String, Arc<PmdPort>>,
    pub rx_queues: HashMap<i32, Vec<CacheAligned<PortQueue>>>,
    pub active_cores: Vec<i32>,
    pub scheduler_channels: HashMap<i32, SyncSender<SchedulerCommand>>,
    pub scheduler_handles: HashMap<i32, JoinHandle<()>>,
}

impl NetBricksContext {
    pub fn start_schedulers(&mut self) {
        let cores = self.active_cores.clone();
        for core in &cores {
            self.start_scheduler(*core);
        }
    }

    #[inline]
    fn start_scheduler(&mut self, core: i32) {
        let builder = thread::Builder::new();
        let (sender, receiver) = sync_channel(0);
        self.scheduler_channels.insert(core, sender);
        let join_handle = builder.name(format!("sched-{}", core).into())
            .spawn(move || {
                init_thread(core, core);
                // Other init?
                let mut sched = Scheduler::new_with_channel(receiver);
                sched.handle_requests()
            })
            .unwrap();
        self.scheduler_handles.insert(core, join_handle);
    }


    pub fn add_pipeline_to_run<T: Fn(Vec<AlignedPortQueue>, &mut Scheduler) + Send + Sync + 'static>(&mut self,
                                                                                                     run: Arc<T>) {
        for (core, channel) in &self.scheduler_channels {
            let ports = match self.rx_queues.get(core) {
                Some(v) => v.clone(),
                None => vec![],
            };
            let boxed_run = run.clone();
            channel.send(SchedulerCommand::Run(Arc::new(move |s| boxed_run(ports.clone(), s)))).unwrap();
        }
    }

    pub fn execute(&mut self) {
        for (core, channel) in &self.scheduler_channels {
            channel.send(SchedulerCommand::Execute).unwrap();
            println!("Starting scheduler on {}", core);
        }
    }
}

/// Initialize the system from a configuration.
pub fn initialize_system(configuration: &NetbricksConfiguration) -> Result<NetBricksContext> {
    init_system(configuration);
    let mut ctx: NetBricksContext = Default::default();
    let mut cores: HashSet<_> = configuration.cores.iter().cloned().collect();
    for port in &configuration.ports {
        if ctx.ports.contains_key(&port.name) {
            println!("Port {} appears twice in specification", port.name);
            return Err(ErrorKind::ConfigurationError(format!("Port {} appears twice in specification", port.name))
                .into());
        } else {
            match PmdPort::new_port_from_configuration(port) {
                Ok(p) => {
                    ctx.ports.insert(port.name.clone(), p);
                }
                Err(e) => {
                    return Err(ErrorKind::ConfigurationError(format!("Port {} could not be initialized {:?}",
                                                                     port.name,
                                                                     e))
                        .into())
                }
            }

            let port_instance = &ctx.ports[&port.name];

            for (rx_q, core) in port.rx_queues.iter().enumerate() {
                let rx_q = rx_q as i32;
                match PmdPort::new_queue_pair(port_instance, rx_q, rx_q) {
                    Ok(q) => {
                        ctx.rx_queues.entry(*core).or_insert_with(|| vec![]).push(q);
                    }
                    Err(e) => {
                        return Err(ErrorKind::ConfigurationError(format!("Queue {} on port {} could not be \
                                                                          initialized {:?}",
                                                                         rx_q,
                                                                         port.name,
                                                                         e))
                            .into())
                    }
                }
            }
        }
    }
    if configuration.strict {
        let other_cores: HashSet<_> = ctx.rx_queues.keys().cloned().collect();
        let core_diff: Vec<_> = other_cores.difference(&cores).map(|c| c.to_string()).collect();
        if !core_diff.is_empty() {
            let missing_str = core_diff.join(", ");
            return Err(ErrorKind::ConfigurationError(format!("Strict configuration selected but core(s) {} appear \
                                                              in port configuration but not in cores",
                                                             missing_str))
                .into());
        }
    } else {
        cores.extend(ctx.rx_queues.keys());
    };
    ctx.active_cores = cores.into_iter().collect();
    Ok(ctx)
}
