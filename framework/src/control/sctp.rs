use super::{Available, HUP, IOScheduler, PollHandle, PollScheduler, READ, Token, WRITE};
use fnv::FnvHasher;
use scheduler::Executable;
/// SCTP Connections.
use sctp::*;

use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::marker::PhantomData;
use std::net::{SocketAddr, ToSocketAddrs};
use std::os::unix::io::AsRawFd;


pub trait SctpControlAgent {
    fn new(address: SocketAddr, stream: SctpStream, scheduler: IOScheduler) -> Self;
    fn handle_read_ready(&mut self) -> bool;
    fn handle_write_ready(&mut self) -> bool;
    fn handle_hup(&mut self) -> bool;
}

type FnvHash = BuildHasherDefault<FnvHasher>;
pub struct SctpControlServer<T: SctpControlAgent> {
    listener: SctpListener,
    scheduler: PollScheduler,
    handle: PollHandle,
    next_token: Token,
    listener_token: Token,
    phantom_t: PhantomData<T>,
    connections: HashMap<Token, T, FnvHash>,
}

impl<T: SctpControlAgent> Executable for SctpControlServer<T> {
    fn execute(&mut self) {
        self.schedule();
    }

    #[inline]
    fn dependencies(&mut self) -> Vec<usize> {
        vec![]
    }
}

// FIXME: Add one-to-many SCTP support?
impl<T: SctpControlAgent> SctpControlServer<T> {
    pub fn new_streaming<A: ToSocketAddrs>(address: A) -> SctpControlServer<T> {
        let listener = SctpListener::bind(address).unwrap();
        let _ = listener.set_nonblocking(true).unwrap();
        let scheduler = PollScheduler::new();
        let listener_token = 0;
        let handle = scheduler.new_poll_handle();
        handle.new_io_port(&listener, listener_token);
        handle.schedule_read(&listener, listener_token);
        SctpControlServer {
            listener: listener,
            scheduler: scheduler,
            handle: handle,
            next_token: listener_token + 1,
            listener_token: listener_token,
            phantom_t: PhantomData,
            connections: HashMap::with_capacity_and_hasher(32, Default::default()),
        }
    }

    fn listen(&mut self) {
        self.handle
            .schedule_read(&self.listener, self.listener_token);
    }

    pub fn schedule(&mut self) {
        match self.scheduler.get_token_noblock() {
            Some((token, avail)) if token == self.listener_token => {
                self.accept_connection(avail);
            }
            Some((token, available)) => {
                self.handle_data(token, available);
            }
            _ => {}
        }
    }

    fn accept_connection(&mut self, available: Available) {
        if available & READ != 0 {
            // Make sure we have something to accept
            match self.listener.accept() {
                Ok((stream, addr)) => {
                    let token = self.next_token;
                    self.next_token += 1;
                    let _ = stream.set_nonblocking(true).unwrap();
                    let stream_fd = stream.as_raw_fd();
                    self.connections
                        .insert(token,
                                T::new(addr,
                                       stream,
                                       IOScheduler::new(self.scheduler.new_poll_handle(), stream_fd, token)));
                    // Add to some sort of hashmap.
                }
                Err(_) => {
                    // FIXME: Record
                }
            }
        } else {
            // FIXME: Report something.
        }
        self.listen();
    }

    fn handle_data(&mut self, token: Token, available: Available) {
        let preserve = {
            match self.connections.get_mut(&token) {
                Some(mut connection) => {
                    if available & READ != 0 {
                        connection.handle_read_ready()
                    } else if available & WRITE != 0 {
                        connection.handle_write_ready()
                    } else if available & HUP != 0 {
                        connection.handle_hup()
                    } else {
                        true
                    }
                }
                None => {
                    // FIXME: Record
                    true
                }
            }
        };

        if !preserve {
            self.connections.remove(&token);
        }
    }
}
