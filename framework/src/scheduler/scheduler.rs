use std::default::Default;
use std::sync::Arc;
use std::sync::mpsc::{SyncSender, Receiver, sync_channel, RecvError};
use std::thread;
use super::Executable;
use utils;

/// Used to keep stats about each pipeline and eventually grant tokens, etc.
struct Runnable {
    pub task: Box<Executable>,
    pub cycles: u64,
    pub last_run: u64,
}

impl Runnable {
    pub fn from_task<T: Executable + 'static>(task: T) -> Runnable {
        Runnable {
            task: box task,
            cycles: 0,
            last_run: utils::rdtsc_unsafe(),
        }
    }
    pub fn from_boxed_task(task: Box<Executable>) -> Runnable {
        Runnable {
            task: task,
            cycles: 0,
            last_run: utils::rdtsc_unsafe(),
        }
    }
}

/// A very simple round-robin scheduler. This should really be more of a DRR scheduler.
pub struct Scheduler {
    /// The set of runnable items. Note we currently don't have a blocked queue.
    run_q: Vec<Runnable>,
    /// Next task to run.
    next_task: usize,
    /// Channel to communicate and synchronize with scheduler.
    sched_channel: Receiver<SchedulerCommand>,
    /// Signal scheduler should continue executing tasks.
    execute_loop: bool,
    /// Signal scheduler should shutdown.
    shutdown: bool,
}

/// Messages that can be sent on the scheduler channel to add or remove tasks.
pub enum SchedulerCommand {
    Add(Box<Executable + Send>),
    Run(Arc<Fn(&mut Scheduler) + Send + Sync>),
    Execute,
    Shutdown,
    Handshake(SyncSender<bool>),
}

const DEFAULT_Q_SIZE: usize = 256;

impl Default for Scheduler {
    fn default() -> Scheduler {
        Scheduler::new()
    }
}

impl Scheduler {
    pub fn new() -> Scheduler {
        let (_, receiver) = sync_channel(0);
        Scheduler::new_with_channel(receiver)
    }

    pub fn new_with_channel(channel: Receiver<SchedulerCommand>) -> Scheduler {
        Scheduler::new_with_channel_and_capacity(channel, DEFAULT_Q_SIZE)
    }

    pub fn new_with_channel_and_capacity(channel: Receiver<SchedulerCommand>, capacity: usize) -> Scheduler {
        Scheduler {
            run_q: Vec::with_capacity(capacity),
            next_task: 0,
            sched_channel: channel,
            execute_loop: false,
            shutdown: true,
        }
    }

    fn handle_request(&mut self, request: SchedulerCommand) {
        match request {
            SchedulerCommand::Add(ex) => self.run_q.push(Runnable::from_boxed_task(ex)),
            SchedulerCommand::Run(f) => f(self),
            SchedulerCommand::Execute => self.execute_loop(),
            SchedulerCommand::Shutdown => {
                self.execute_loop = false;
                self.shutdown = true;
            }
            SchedulerCommand::Handshake(chan) => {
                chan.send(true).unwrap(); // Inform context about reaching barrier.
                thread::park();
            }
        }
    }

    pub fn handle_requests(&mut self) {
        self.shutdown = false;
        // Note this rather bizarre structure here to get shutting down hooked in.
        while let Ok(cmd) = {
            if self.shutdown {
                Err(RecvError)
            } else {
                self.sched_channel.recv()
            }
        } {
            self.handle_request(cmd)
        }
        println!("Scheduler exiting {}",
                 thread::current().name().unwrap_or_else(|| "unknown-name"));
    }

    /// Add a task to the current scheduler.
    pub fn add_task<T: Executable + 'static>(&mut self, task: T) {
        self.run_q.push(Runnable::from_task(task))
    }

    #[inline]
    fn execute_internal(&mut self) {
        {
            let task = &mut (&mut self.run_q[self.next_task]);
            let begin = utils::rdtsc_unsafe();
            task.task.execute();
            let end = utils::rdtsc_unsafe();
            task.cycles += end - begin;
            task.last_run = end;
        }
        let len = self.run_q.len();
        let next = self.next_task + 1;
        if next == len {
            self.next_task = 0;
            if let Ok(cmd) = self.sched_channel.try_recv() {
                self.handle_request(cmd);
            }
        } else {
            self.next_task = next;
        }
    }

    /// Run the scheduling loop.
    pub fn execute_loop(&mut self) {
        self.execute_loop = true;
        if !self.run_q.is_empty() {
            while self.execute_loop {
                self.execute_internal()
            }
        }
    }

    pub fn execute_one(&mut self) {
        if !self.run_q.is_empty() {
            self.execute_internal()
        }
    }
}
