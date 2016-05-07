extern crate time;
use super::Executable;
use std::cell::RefCell;
/// A very simple round-robin scheduler. This should really be more of a DRR scheduler.
pub struct Scheduler {
    /// The set of runnable items. Note we currently don't have a blocked queue.
    // FIXME: Consider making this a linked list, in which case next_task is unnecessary?
    run_q: Vec<RefCell<Box<Executable>>>,
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
    pub fn add_task(&mut self, task: RefCell<Box<Executable>>) {
        self.run_q.push(task)
    }

    #[inline]
    fn execute_internal(&mut self) {
        let mut task = self.run_q[self.next_task].borrow_mut();
        let next = self.next_task + 1;
        if next == self.run_q.len() {
            self.next_task = 0;
        } else {
            self.next_task = next
        }
        task.execute()
    }

    /// Run the scheduling loop.
    // TODO: Add a variable to stop the scheduler (for whatever reason).
    #[inline]
    pub fn execute_loop(&mut self) {
        if !self.run_q.is_empty() {
            loop {self.execute_internal()}
        }
    }

    #[inline]
    pub fn execute_one(&mut self) {
        if !self.run_q.is_empty() {
            self.execute_internal()
        }
    }

    #[inline]
    pub fn execute_loop_timed(&mut self, pfx: &str, batch: usize) {
        if !self.run_q.is_empty() {
            loop {
                let start = time::precise_time_ns();
                for _ in 0..batch {
                    self.execute_internal()
                }
                let end = time::precise_time_ns();
                println!("{} Time for {} batches was {}", pfx, batch, end - start);
            }
        }
    }
}
