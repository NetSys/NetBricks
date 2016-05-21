use super::Executable;
/// A very simple round-robin scheduler. This should really be more of a DRR scheduler.
pub struct Scheduler {
    /// The set of runnable items. Note we currently don't have a blocked queue.
    run_q: Vec<Box<Executable>>,
    /// Next task to run.
    next_task: usize,
}

const DEFAULT_Q_SIZE: usize = 256;

impl Scheduler {
    pub fn new() -> Scheduler {
        Scheduler {
            run_q: Vec::with_capacity(DEFAULT_Q_SIZE),
            next_task: 0,
        }
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
