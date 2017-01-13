use std;

use e2d2::common::*;
use e2d2::interface::*;
use e2d2::operators::*;
use e2d2::scheduler::embedded_scheduler::*;
use e2d2::native::zcsi::*;
use nf;

#[repr(C)]
pub struct PacketBuf {
    pub capacity: usize,
    pub cnt: usize,
    pub pkts: *mut *mut mbuf::MBuf,
}

#[derive(Clone)]
struct Cookie {
    qid: i32,
    rx_buf: *mut PacketBuf,
    tx_buf: *mut PacketBuf,
}

unsafe impl Send for Cookie {}
unsafe impl Sync for Cookie {}

pub struct NetbricksBessMod {
    sched: EmbeddedScheduler,
    task_ids: Vec<usize>,
    port: std::sync::Arc<CallbackPort<Cookie>>,
}

#[inline]
fn recv(cookie: &Cookie, pkts: &mut [*mut mbuf::MBuf]) -> Result<u32> {
    unsafe {
        let ref mut rx_buf = *cookie.rx_buf;
        let cnt = rx_buf.cnt;

        std::ptr::copy_nonoverlapping(rx_buf.pkts, pkts.as_mut_ptr(), cnt);
        rx_buf.cnt = 0;

        Ok(cnt as u32)
    }
}

#[inline]
fn send(cookie: &Cookie, pkts: &mut [*mut mbuf::MBuf]) -> Result<u32> {
    unsafe {
        let ref mut tx_buf = *cookie.tx_buf;
        let cnt = pkts.len();
        let to_copy = std::cmp::min(cnt, tx_buf.capacity - tx_buf.cnt);

        std::ptr::copy_nonoverlapping(pkts.as_mut_ptr(),
                                      tx_buf.pkts.offset(tx_buf.cnt as isize),
                                      to_copy);
        tx_buf.cnt += to_copy;

        if cnt > to_copy {
            mbuf_free_bulk(pkts.as_mut_ptr().offset(to_copy as isize),
                           (cnt - to_copy) as i32);
        }

        Ok(cnt as u32)
    }
}

#[no_mangle]
pub extern fn init_mod(rx_buf: *mut PacketBuf, tx_buf: *mut PacketBuf) -> *mut NetbricksBessMod {
    let cookie = Cookie {
        qid: 0,
        rx_buf: rx_buf,
        tx_buf: tx_buf,
    };

    let mut sched = EmbeddedScheduler::new();
    let port = CallbackPort::new(1, recv, send).unwrap();
    let queue = port.new_callback_queue(cookie).unwrap();

    let mut task_ids = Vec::<usize>::new();

    // Replace this with your own pipeline. Repeat --------------------------
    let id = sched.add_task(nf::delay(ReceiveBatch::new(queue.clone()), 1).send(queue.clone()));
    task_ids.push(id.unwrap());
    // ----------------------------------------------------------------------

    let ctx = Box::new(NetbricksBessMod {
        sched: sched,
        task_ids: task_ids,
        port: port,
    });

    return Box::into_raw(ctx);
}

#[no_mangle]
pub extern fn deinit_mod(_ctx: *mut NetbricksBessMod) {
    unsafe {
        let ctx = Box::from_raw(_ctx);
        drop(ctx);      // unnecessary, just to avoid 'unused variable' warning
    }
}

#[no_mangle]
pub extern fn run_once(_ctx: *mut NetbricksBessMod) {
    let ctx: &mut NetbricksBessMod = unsafe { _ctx.as_mut().unwrap() };

    for id in &ctx.task_ids {
        ctx.sched.exec_task(*id);
    }
}

#[no_mangle]
pub extern fn get_stats(_ctx: *mut NetbricksBessMod) -> (usize, usize) {
    let ctx: &mut NetbricksBessMod = unsafe { _ctx.as_mut().unwrap() };
    ctx.port.stats()
}
