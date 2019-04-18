use netbricks::control::tcp::*;
use std::net::*;
use std::io::Read;
use nix::errno;

pub struct ControlListener {
    scheduler: TcpScheduler,
    stream: TcpStream,
    buffer: Vec<u8>,
    read_till: usize,
}
impl TcpControlAgent for ControlListener {
    fn new(address: SocketAddr, stream: TcpStream, scheduler: TcpScheduler) -> ControlListener {
        println!("New connection from {}", address);
        scheduler.schedule_read();
        ControlListener { scheduler: scheduler, stream: stream, buffer: (0..14).map(|_| 0).collect(), read_till: 0 }
    }

    fn handle_read_ready(&mut self) -> bool {
        let mut schedule = true;
        while {
            let read_till = self.read_till;
            let r = self.stream.read(&mut self.buffer[read_till..]);
            match r {
                Ok(r) => {
                    if r > 0 {
                        if read_till + r == 14 {
                            //println!("Complete message");
                            self.read_till = 0;
                        }
                    };
                    r > 0
                },
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
                },
            }
        } {
        }
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
