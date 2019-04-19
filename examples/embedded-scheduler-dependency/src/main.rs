#![feature(box_syntax)]
#![feature(asm)]
extern crate fnv;
extern crate netbricks;
extern crate rand;
extern crate time;
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
        let mut nhandles: Vec<_> = (0..10).map(|_| 0).collect();
        for i in 0..nhandles.capacity() {
            nhandles[i] = sched
                .add_task(DepTask::new(prev_handle, format!("id-{}", i).as_str()))
                .unwrap();
            prev_handle = nhandles[i];
        }
        nhandles
    };
    let len = other_handles.len();
    sched.exec_task(other_handles[len - 1]);
}
