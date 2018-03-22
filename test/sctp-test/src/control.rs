use netbricks::control::IOScheduler;
use netbricks::control::sctp::*;
use nix::errno;
use sctp::*;
use std::net::SocketAddr;

pub struct ControlListener {
    scheduler: IOScheduler,
    stream: SctpStream,
    buffer: Vec<u8>,
}
impl SctpControlAgent for ControlListener {
    fn new(address: SocketAddr, stream: SctpStream, scheduler: IOScheduler) -> ControlListener {
        println!("New connection from {}", address);
        scheduler.schedule_read();
        ControlListener {
            scheduler: scheduler,
            stream: stream,
            buffer: (0..1024).map(|_| 0).collect(),
        }
    }

    fn handle_read_ready(&mut self) -> bool {
        let mut schedule = true;
        while {
            let read = self.stream.recvmsg(&mut self.buffer[..]);
            match read {
                Ok((size, stream)) => {
                    println!("Received message on stream {} of size {}", stream, size);
                    true
                }
                Err(e) => {
                    if let Some(e) = e.raw_os_error() {
                        if errno::from_i32(e) != errno::Errno::EAGAIN {
                            schedule = false;
                        } else {
                            schedule = true;
                        }
                    } else {
                        schedule = false;
                    }
                    false
                }
            }
        } {}
        if schedule {
            self.scheduler.schedule_read();
        };
        schedule
    }

    fn handle_write_ready(&mut self) -> bool {
        panic!("No writes expected");
    }

    fn handle_hup(&mut self) -> bool {
        println!("Hanging up");
        false
    }
}
