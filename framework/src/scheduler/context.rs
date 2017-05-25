use allocators::CacheAligned;
use config::NetbricksConfiguration;
use interface::{PmdPort, PortQueue, VirtualQueue, VirtualPort};
use interface::dpdk::{init_system, init_thread};
use scheduler::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::mpsc::{SyncSender, sync_channel};
use std::thread::{self, JoinHandle, Thread};

type AlignedPortQueue = CacheAligned<PortQueue>;
type AlignedVirtualQueue = CacheAligned<VirtualQueue>;

/// A handle to schedulers paused on a barrier.
pub struct BarrierHandle<'a> {
    threads: Vec<&'a Thread>,
}

impl<'a> BarrierHandle<'a> {
    /// Release all threads. This consumes the handle as expected.
    pub fn release(self) {
        for thread in &self.threads {
            thread.unpark();
        }
    }

    /// Allocate a new BarrierHandle with threads.
    pub fn with_threads(threads: Vec<&'a Thread>) -> BarrierHandle {
        BarrierHandle { threads: threads }
    }
}

/// `NetBricksContext` contains handles to all schedulers, and provides mechanisms for coordination.
#[derive(Default)]
pub struct NetBricksContext {
    pub ports: HashMap<String, Arc<PmdPort>>,
    pub rx_queues: HashMap<i32, Vec<CacheAligned<PortQueue>>>,
    pub active_cores: Vec<i32>,
    pub virtual_ports: HashMap<i32, Arc<VirtualPort>>,
    scheduler_channels: HashMap<i32, SyncSender<SchedulerCommand>>,
    scheduler_handles: HashMap<i32, JoinHandle<()>>,
}

impl NetBricksContext {
    /// Boot up all schedulers.
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
        let join_handle = builder
            .name(format!("sched-{}", core).into())
            .spawn(move || {
                       init_thread(core, core);
                       // Other init?
                       let mut sched = StandaloneScheduler::new_with_channel(receiver);
                       sched.handle_requests()
                   })
            .unwrap();
        self.scheduler_handles.insert(core, join_handle);
    }

    /// Run a function (which installs a pipeline) on all schedulers in the system.
    pub fn add_pipeline_to_run<T>(&mut self, run: Arc<T>)
        where T: Fn(Vec<AlignedPortQueue>, &mut StandaloneScheduler) + Send + Sync + 'static
    {
        for (core, channel) in &self.scheduler_channels {
            let ports = match self.rx_queues.get(core) {
                Some(v) => v.clone(),
                None => vec![],
            };
            let boxed_run = run.clone();
            channel
                .send(SchedulerCommand::Run(Arc::new(move |s| boxed_run(ports.clone(), s))))
                .unwrap();
        }
    }

    pub fn add_test_pipeline<T>(&mut self, run: Arc<T>)
        where T: Fn(Vec<AlignedVirtualQueue>, &mut StandaloneScheduler) + Send + Sync + 'static
    {
        for (core, channel) in &self.scheduler_channels {
            let port = self.virtual_ports
                .entry(*core)
                .or_insert(VirtualPort::new(1).unwrap());
            let boxed_run = run.clone();
            let queue = port.new_virtual_queue(1).unwrap();
            channel
                .send(SchedulerCommand::Run(Arc::new(move |s| boxed_run(vec![queue.clone()], s))))
                .unwrap();
        }
    }

    pub fn add_test_pipeline_to_core<T: Fn(Vec<AlignedVirtualQueue>, &mut StandaloneScheduler) + Send + Sync + 'static>
        (&mut self,
         core: i32,
         run: Arc<T>)
         -> Result<()> {

        if let Some(channel) = self.scheduler_channels.get(&core) {
            let port = self.virtual_ports
                .entry(core)
                .or_insert(VirtualPort::new(1).unwrap());
            let boxed_run = run.clone();
            let queue = port.new_virtual_queue(1).unwrap();
            channel
                .send(SchedulerCommand::Run(Arc::new(move |s| boxed_run(vec![queue.clone()], s))))
                .unwrap();
            Ok(())
        } else {
            Err(ErrorKind::NoRunningSchedulerOnCore(core).into())
        }
    }

    /// Install a pipeline on a particular core.
    pub fn add_pipeline_to_core<T: Fn(Vec<AlignedPortQueue>, &mut StandaloneScheduler) + Send + Sync + 'static>
        (&mut self,
         core: i32,
         run: Arc<T>)
         -> Result<()> {
        if let Some(channel) = self.scheduler_channels.get(&core) {
            let ports = match self.rx_queues.get(&core) {
                Some(v) => v.clone(),
                None => vec![],
            };
            let boxed_run = run.clone();
            channel
                .send(SchedulerCommand::Run(Arc::new(move |s| boxed_run(ports.clone(), s))))
                .unwrap();
            Ok(())
        } else {
            Err(ErrorKind::NoRunningSchedulerOnCore(core).into())
        }
    }

    /// Start scheduling pipelines.
    pub fn execute(&mut self) {
        for (core, channel) in &self.scheduler_channels {
            channel.send(SchedulerCommand::Execute).unwrap();
            println!("Starting scheduler on {}", core);
        }
    }

    /// Pause all schedulers, the returned `BarrierHandle` can be used to resume.
    pub fn barrier(&mut self) -> BarrierHandle {
        // TODO: If this becomes a problem, move this to the struct itself; but make sure to fix `stop` appropriately.
        let channels: Vec<_> = self.scheduler_handles
            .iter()
            .map(|_| sync_channel(0))
            .collect();
        let receivers = channels.iter().map(|&(_, ref r)| r);
        let senders = channels.iter().map(|&(ref s, _)| s);
        for ((_, channel), sender) in self.scheduler_channels.iter().zip(senders) {
            channel
                .send(SchedulerCommand::Handshake(sender.clone()))
                .unwrap();
        }
        for receiver in receivers {
            receiver.recv().unwrap();
        }
        BarrierHandle::with_threads(self.scheduler_handles
                                        .values()
                                        .map(|j| j.thread())
                                        .collect())
    }

    /// Stop all schedulers, safely shutting down the system.
    pub fn stop(&mut self) {
        for (core, channel) in &self.scheduler_channels {
            channel.send(SchedulerCommand::Shutdown).unwrap();
            println!("Issued shutdown for core {}", core);
        }
        for (core, join_handle) in self.scheduler_handles.drain() {
            join_handle.join().unwrap();
            println!("Core {} has shutdown", core);
        }
        println!("System shutdown");
    }

    pub fn wait(&mut self) {
        for (core, join_handle) in self.scheduler_handles.drain() {
            join_handle.join().unwrap();
            println!("Core {} has shutdown", core);
        }
        println!("System shutdown");
    }

    /// Shutdown all schedulers.
    pub fn shutdown(&mut self) {
        self.stop()
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
        let core_diff: Vec<_> = other_cores
            .difference(&cores)
            .map(|c| c.to_string())
            .collect();
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
