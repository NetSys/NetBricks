#![feature(box_syntax)]
#![feature(asm)]
extern crate e2d2;
extern crate fnv;
extern crate time;
extern crate rand;
use e2d2::scheduler::*;

fn test_func(id: &str) {
    println!("Running function {}", id);
}

fn main() {
    let mut sched = embedded_scheduler::EmbeddedScheduler::new();
    let handle0 = sched.add_task(|| test_func("task-0")).unwrap();
    let handle1 = sched.add_task(|| test_func("task-1")).unwrap();
    println!("Initialized");
    sched.exec_task(handle1);
    sched.exec_task(handle0);
}
