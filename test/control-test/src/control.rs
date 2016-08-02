use e2d2::control::tcp::*;
use std::net::*;
use std::io::Read;

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
        while {
            let r = self.stream.read(&mut self.buffer[..]).unwrap();
            r > 0
        } {
        }
        self.scheduler.schedule_read();
        true
    }
    
    fn handle_write_ready(&mut self) -> bool {
        panic!("No writes expected");
    }
    
    fn handle_hup(&mut self) -> bool {
        false
    }
}
