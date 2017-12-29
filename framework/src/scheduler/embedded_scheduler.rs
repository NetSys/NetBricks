use super::{Executable, Scheduler};
use common::*;
use std::default::Default;

/// Used to keep stats about each pipeline and eventually grant tokens, etc.
struct Runnable {
    pub task: Box<Executable>,
    pub dependencies: Vec<usize>,
}

impl Runnable {
    pub fn from_task<T: Executable + 'static>(mut task: T) -> Runnable {
        let deps = task.dependencies();
        Runnable {
            task: box task,
            dependencies: deps,
        }
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

impl Scheduler for EmbeddedScheduler {
    /// Add a task, and return a handle allowing the task to be run.
    fn add_task<T: Executable + 'static>(&mut self, task: T) -> Result<usize> {
        self.tasks.push(Runnable::from_task(task));
        Ok(self.tasks.len())
    }
}

impl EmbeddedScheduler {
    /// Create a new Bess scheduler.
    pub fn new() -> EmbeddedScheduler {
        EmbeddedScheduler {
            tasks: Vec::with_capacity(DEFAULT_TASKQ_SIZE),
        }
    }

    /// Run specified task.
    pub fn exec_task(&mut self, task_id: usize) {
        {
            let len = self.tasks[task_id - 1].dependencies.len();
            for dep in 0..len {
                let dep_task = self.tasks[task_id - 1].dependencies[dep];
                self.exec_task(dep_task)
            }
        }
        self.tasks[task_id - 1].task.execute();
    }

    fn display_dependencies_internal(&self, task_id: usize, depth: usize) {
        {
            let len = self.tasks[task_id - 1].dependencies.len();
            for dep in 0..len {
                let dep_task = self.tasks[task_id - 1].dependencies[dep];
                self.display_dependencies_internal(dep_task, depth + 1)
            }
        }
        println!("{} Task {}", depth, task_id);
    }

    /// For debugging purposes
    pub fn display_dependencies(&mut self, task_id: usize) {
        self.display_dependencies_internal(task_id, 0)
    }
}
