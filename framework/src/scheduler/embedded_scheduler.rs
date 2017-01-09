use common::*;
use std::default::Default;
use super::Executable;

/// Used to keep stats about each pipeline and eventually grant tokens, etc.
struct Runnable {
    pub task: Box<Executable>,
}

impl Runnable {
    pub fn from_task<T: Executable + 'static>(task: T) -> Runnable {
        Runnable { task: box task }
    }
}

/// This scheduler is designed to allow NetBricks to be embedded in other vswitches (e.g., Bess). As a result it neither
/// does any of the resource accounting `Scheduler` attempts to do at the moment, nor does it have anything that just
/// runs tasks in a loop.
pub struct EmbeddedScheduler {
    /// The set of runnable items. Note we currently don't have a blocked queue.
    tasks: Vec<Runnable>,
}

const DEFAULT_TASKQ_SIZE: usize = 256;

impl Default for EmbeddedScheduler {
    fn default() -> EmbeddedScheduler {
        EmbeddedScheduler::new()
    }
}

impl EmbeddedScheduler {
    /// Create a new Bess scheduler.
    pub fn new() -> EmbeddedScheduler {
        EmbeddedScheduler { tasks: Vec::with_capacity(DEFAULT_TASKQ_SIZE) }
    }

    /// Add a task, and return a handle allowing the task to be run.
    pub fn add_task<T: Executable + 'static>(&mut self, task: T) -> Result<usize> {
        self.tasks.push(Runnable::from_task(task));
        Ok(self.tasks.len())
    }

    /// Run specified task.
    pub fn exec_task(&mut self, task_id: usize) {
        self.tasks[task_id - 1].task.execute();
    }
}
