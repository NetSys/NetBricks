use std::sync::mpsc::{sync_channel, Receiver};
use std::sync::Arc;
use super::Executable;
/// A very simple round-robin scheduler. This should really be more of a DRR scheduler.
pub struct Scheduler {
    /// The set of runnable items. Note we currently don't have a blocked queue.
    run_q: Vec<Box<Executable>>,
    /// Next task to run.
    next_task: usize,
    sched_channel: Receiver<SchedulerCommand>,
}

pub enum SchedulerCommand {
    Add(Box<Executable + Send>),
    Run(Arc<Fn(&mut Scheduler) + Send + Sync>),
    Execute,
}

const DEFAULT_Q_SIZE: usize = 256;

impl Scheduler {
    pub fn new() -> Scheduler {
        let (_, receiver) = sync_channel(0);
        Scheduler::new_with_channel(receiver)
    }

    pub fn new_with_channel(channel: Receiver<SchedulerCommand>) -> Scheduler {
        Scheduler {
            run_q: Vec::with_capacity(DEFAULT_Q_SIZE),
            next_task: 0,
            sched_channel: channel,
        }
    }

    fn handle_request(&mut self, request: SchedulerCommand) {
        match request {
            SchedulerCommand::Add(ex) => self.run_q.push(ex),
            SchedulerCommand::Run(f) => { f(self) },
            SchedulerCommand::Execute => { self.execute_loop() }
        }
    }

    pub fn handle_requests(&mut self) {
        loop {
            match self.sched_channel.recv() {
                Ok(cmd) => {
                    self.handle_request(cmd)
                },
                _ => {
                    break
                }
            }
        }
        println!("Scheduler exiting");
    }

    /// Add a task to the current scheduler.
    pub fn add_task<T: Executable + 'static>(&mut self, task: T) {
        self.run_q.push(box task)
    }

    #[inline]
    fn execute_internal(&mut self) {
        let len = self.run_q.len();
        let ref mut task = &mut self.run_q[self.next_task];
        let next = self.next_task + 1;
        if next == len {
            self.next_task = 0;
        } else {
            self.next_task = next
        }
        task.execute()
    }

    /// Run the scheduling loop.
    // TODO: Add a variable to stop the scheduler (for whatever reason).
    pub fn execute_loop(&mut self) {
        if !self.run_q.is_empty() {
            loop {
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
