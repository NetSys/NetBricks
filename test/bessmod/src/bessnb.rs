use std;

use e2d2::common::*;
use e2d2::interface::*;
use e2d2::operators::*;
use e2d2::scheduler::embedded_scheduler::*;
use e2d2::native::zcsi::mbuf::MBuf;
use nf;

#[repr(C)]
pub struct BessGate {
    pub capacity: usize,
    pub cnt: usize,
    pub pkts: *mut *mut MBuf,
}

#[repr(C)]
pub struct NetbricksBessMod {
    sched: EmbeddedScheduler,
    task_ids: Vec<usize>,
    port: std::sync::Arc<CallbackPort<Cookie>>,
}

#[derive(Clone)]
struct Cookie {
    qid: usize,
    rx_buf: *mut BessGate,
    tx_buf: *mut BessGate,
}

unsafe impl Send for Cookie {}
unsafe impl Sync for Cookie {}

#[inline]
fn recv(cookie: &Cookie, pkts: &mut [*mut MBuf]) -> Result<u32> {
    unsafe {
        let rx_buf = &mut *cookie.rx_buf;
        let cnt = rx_buf.cnt;

        std::ptr::copy_nonoverlapping(rx_buf.pkts, pkts.as_mut_ptr(), cnt);
        rx_buf.cnt = 0;

        Ok(cnt as u32)
    }
}

#[inline]
fn send(cookie: &Cookie, pkts: &mut [*mut MBuf]) -> Result<u32> {
    unsafe {
        let tx_buf = &mut *cookie.tx_buf;
        let cnt = pkts.len();
        let to_copy = std::cmp::min(cnt, tx_buf.capacity - tx_buf.cnt);

        std::ptr::copy_nonoverlapping(pkts.as_mut_ptr(),
                                      tx_buf.pkts.offset(tx_buf.cnt as isize),
                                      to_copy);
        tx_buf.cnt += to_copy;

        Ok(cnt as u32)
    }
}

#[no_mangle]
pub extern "C" fn init_mod(num_gates: usize, rx_bufs: *mut *mut BessGate, tx_bufs: *mut *mut BessGate) -> *mut NetbricksBessMod {
    let mut sched = EmbeddedScheduler::new();
    let port = CallbackPort::new(num_gates as i32, recv, send).unwrap();

    let mut gates = Vec::new();

    for qid in 0..num_gates {
        let cookie;

        unsafe {
            cookie = Cookie {
                qid: qid,
                rx_buf: (*rx_bufs.offset(qid as isize)),
                tx_buf: (*tx_bufs.offset(qid as isize)),
            };
        }

        gates.push(port.new_callback_queue(cookie).unwrap());
    }

    let mut task_ids = Vec::<usize>::new();

    // With a given "gates", register your pipelines as necessary -----------
    // Collect task IDs returned by sched.add_task(). Below is an example.
    for gate in gates {
        let rx_gate = gate.clone();
        let tx_gate = gate.clone();
        let id = sched.add_task(nf::delay(ReceiveBatch::new(rx_gate), 1).send(tx_gate));
        task_ids.push(id.unwrap());
    }
    // ----------------------------------------------------------------------

    let ctx = Box::new(NetbricksBessMod {
        sched: sched,
        task_ids: task_ids,
        port: port,
    });

    return Box::into_raw(ctx);
}

#[no_mangle]
pub extern "C" fn deinit_mod(_ctx: *mut NetbricksBessMod) {
    unsafe {
        let ctx = Box::from_raw(_ctx);
        drop(ctx);      // unnecessary, just to avoid 'unused variable' warning
    }
}

#[no_mangle]
pub extern "C" fn run_once(_ctx: *mut NetbricksBessMod) {
    let ctx: &mut NetbricksBessMod = unsafe { _ctx.as_mut().unwrap() };

    for id in &ctx.task_ids {
        ctx.sched.exec_task(*id);
    }
}

#[no_mangle]
pub extern "C" fn get_stats(_ctx: *mut NetbricksBessMod) -> (usize, usize) {
    let ctx: &mut NetbricksBessMod = unsafe { _ctx.as_mut().unwrap() };
    ctx.port.stats()
}
