use std::thread::{self, JoinHandle};
use std::sync::mpsc::{SyncSender, sync_channel};
use interface::{PmdPort, PortQueue};
use interface::dpdk::{init_system, init_thread};
use std::sync::Arc;
use std::collections::HashMap;
use std::convert::From;
use scheduler::*;
use super::{ConfigurationError, ConfigurationResult, NetbricksConfiguration};

#[derive(Default)]
pub struct NetBricksContext {
    pub ports: HashMap<String, Arc<PmdPort>>,
    pub rx_queues: HashMap<i32, Vec<PortQueue>>,
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

    pub fn add_pipeline_to_run<T: Fn(Vec<PortQueue>, &mut Scheduler) + Send + Sync + 'static>(&mut self, run: Arc<T>) {
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
pub fn initialize_system(configuration: &NetbricksConfiguration) -> ConfigurationResult<NetBricksContext> {
    init_system(configuration);
    let mut ctx: NetBricksContext = Default::default();
    for port in &configuration.ports {
        if ctx.ports.contains_key(&port.name) {
            println!("Port {} appears twice in specification", port.name);
            return Err(ConfigurationError::from(format!("Port {} appears twice in specification", port.name)));
        } else {
            match PmdPort::new_port_from_configuration(port) {
                Ok(p) => {
                    ctx.ports.insert(port.name.clone(), p);
                }
                Err(e) => {
                    return Err(ConfigurationError::from(format!("Port {} could not be initialized {:?}", port.name, e)))
                }
            }

            let port_instance = ctx.ports.get(&port.name).unwrap();

            for (rx_q, core) in port.rx_queues.iter().enumerate() {
                let rx_q = rx_q as i32;
                match PmdPort::new_queue_pair(port_instance, rx_q, rx_q) {
                    Ok(q) => {
                        ctx.rx_queues.entry(*core).or_insert_with(|| vec![]).push(q);
                    }
                    Err(e) => {
                        return Err(ConfigurationError::from(format!("Queue {} on port {} could not be initialized \
                                                                     {:?}",
                                                                    rx_q,
                                                                    port.name,
                                                                    e)))
                    }
                }
            }
        }
    }
    ctx.active_cores = ctx.rx_queues.keys().cloned().collect();
    Ok(ctx)
}
