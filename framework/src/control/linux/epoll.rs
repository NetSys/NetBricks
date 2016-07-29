use nix::sys::epoll::*;
use std::os::unix::io::AsRawFd;
use std::os::unix::io::RawFd;
use std::slice;
use super::{Available, NONE, WRITE, READ, HUP}; 

pub type Token = u64;

pub struct PollScheduler {
    epoll_fd: RawFd,
    ready_tokens: Vec<EpollEvent>,
    events: usize,
}

impl PollScheduler {
    pub fn new() -> PollScheduler {
        PollScheduler {
            epoll_fd: epoll_create().unwrap(),
            ready_tokens: Vec::with_capacity(32),
            events: 0,
        }
    }

    /// This assumes file is already set to be non-blocking. This must also be called only the first time round.
    pub fn new_io_port<Fd: AsRawFd>(&self, file: &Fd, token: Token) {
        self.new_io_fd(file.as_raw_fd(), token);
    }

    pub fn new_io_fd(&self, fd: RawFd, token: Token) {
        let mut kind = EpollEventKind::empty();
        kind.insert(EPOLLET); // Edge triggered.
        kind.insert(EPOLLONESHOT); // One shot
        let event = EpollEvent {
            events: kind,
            data: token
        };
        let _ = epoll_ctl(self.epoll_fd, EpollOp::EpollCtlAdd, fd, &event).unwrap();
    }

    pub fn schedule_read<Fd:AsRawFd>(&self, file: &Fd, token: Token) {
        self.schedule_read_rawfd(file.as_raw_fd(), token);
    }

    pub fn schedule_read_rawfd(&self, fd: RawFd, token: Token) {
        let mut kind = EpollEventKind::empty();
        kind.insert(EPOLLIN); // Want to receive input
        kind.insert(EPOLLET); // Edge triggered.
        kind.insert(EPOLLONESHOT); // One shot
        let event = EpollEvent {
            events: kind,
            data: token
        };
        let _ = epoll_ctl(self.epoll_fd, EpollOp::EpollCtlMod, fd, &event).unwrap();
    }

    pub fn schedule_write<Fd:AsRawFd>(&self, file: &Fd, token: Token) {
        self.schedule_write_rawfd(file.as_raw_fd(), token);
    }

    pub fn schedule_write_rawfd(&self, fd: RawFd, token: Token) {
        let mut kind = EpollEventKind::empty();
        kind.insert(EPOLLOUT); // Want to receive input
        kind.insert(EPOLLET); // Edge triggered.
        kind.insert(EPOLLONESHOT); // One shot
        let event = EpollEvent {
            events: kind,
            data: token
        };
        let _ = epoll_ctl(self.epoll_fd, EpollOp::EpollCtlMod, fd, &event).unwrap();
    }

    #[inline]
    fn epoll_kind_to_available(&self, kind: &EpollEventKind) -> Available {
        let mut available = NONE;
        if kind.contains(EPOLLIN) {
            available |= READ
        };
        if kind.contains(EPOLLOUT) {
            available |= WRITE
        };
        if kind.contains(EPOLLHUP) || kind.contains(EPOLLERR) {
            available |= HUP
        };
        available
    }

    pub fn get_token_noblock(&mut self) -> Option<(Token, Available)> {
        if self.events > 0 {
            self.events -= 1;
            self.ready_tokens.pop()
        } else {
            let dest = unsafe { slice::from_raw_parts_mut(self.ready_tokens.as_mut_ptr(),
                                                          self.ready_tokens.capacity()) };
            self.events = epoll_wait(self.epoll_fd, dest, 0).unwrap();
            unsafe { self.ready_tokens.set_len(self.events) };
            self.ready_tokens.pop()
        }.map(|t| (t.data, self.epoll_kind_to_available(&t.events)))
    }
}
