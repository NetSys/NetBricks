/// All projects involve building a thread pool. This is the task equivalent for
/// the threadpool in `NetBricks`.
///
/// Anything that implements Runnable can be polled by the scheduler. This thing
/// can be a `Batch` (e.g., `SendBatch`) or something else (e.g., the `GroupBy`
/// operator).
use common::*;
use failure::Fail;

pub use self::context::*;
pub use self::standalone_scheduler::*;

mod context;
pub mod embedded_scheduler;
mod standalone_scheduler;

/// Errors related to schedulers/scheduling
// TODO: extend this, as we probably want more error handling over
//       scheduling
#[derive(Debug, Fail)]
pub enum SchedulerError {
    #[fail(display = "No scheduler running on core {}", _0)]
    NoRunningSchedulerOnCore(i32),
}

pub trait Executable {
    fn execute(&mut self);
    fn dependencies(&mut self) -> Vec<usize>;
}

impl<F> Executable for F
where
    F: FnMut(),
{
    fn execute(&mut self) {
        (*self)()
    }

    fn dependencies(&mut self) -> Vec<usize> {
        vec![]
    }
}

pub trait Scheduler {
    fn add_task<T: Executable + 'static>(&mut self, task: T) -> Result<usize>
    where
        Self: Sized;
}
