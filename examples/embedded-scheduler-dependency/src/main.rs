extern crate netbricks;
use netbricks::scheduler::*;

pub struct DepTask {
    id: String,
    deps: Vec<usize>,
}

impl Executable for DepTask {
    fn execute(&mut self) {
        println!("Task -- {}", self.id);
    }
    fn dependencies(&mut self) -> Vec<usize> {
        self.deps.clone()
    }
}
impl DepTask {
    pub fn new(parent: usize, id: &str) -> DepTask {
        DepTask {
            id: String::from(id),
            deps: vec![parent],
        }
    }
}

fn test_func(id: &str) {
    println!("Base Task -- {}", id);
}

fn main() {
    let mut sched = embedded_scheduler::EmbeddedScheduler::new();
    let handle0 = sched.add_task(|| test_func("task-0")).unwrap();
    let other_handles = {
        let mut prev_handle = handle0;
        let mut nhandles = vec![];

        for i in 0..10 {
            let task = sched
                .add_task(DepTask::new(prev_handle, format!("id-{}", i).as_str()))
                .unwrap();
            nhandles.push(task);
            prev_handle = task;
        }
        nhandles
    };
    let len = other_handles.len();
    sched.exec_task(other_handles[len - 1]);
}
