use e2d2::control::tcp::*;
use std::net::*;
use std::io::Read;
use nix::errno;
use std::str::from_utf8;

pub struct ControlListener {
    scheduler: TcpScheduler,
    stream: TcpStream,
    buffer: Vec<u8>,
}
impl TcpControlAgent for ControlListener {
    fn new(address: SocketAddr, stream: TcpStream, scheduler: TcpScheduler) -> ControlListener {
        println!("New connection from {}", address);
        scheduler.schedule_read();
        ControlListener { scheduler: scheduler, stream: stream, buffer: (0..1024).map(|_| 0).collect() }
    }

    fn handle_read_ready(&mut self) -> bool {
        let mut schedule = true;
        while {
            let r = self.stream.read(&mut self.buffer[..]);
            match r {
                Ok(r) => { 
                    if r > 0 {
                        println!("Read {}", from_utf8(&self.buffer[..r]).unwrap());
                    };
                    r > 0
                },
                Err(e) => {
                    println!("Error {}", e);
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
