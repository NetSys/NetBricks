#[cfg(target_os = "linux")]
pub use self::epoll::*;

#[cfg(target_os = "linux")]
#[path="linux/epoll.rs"] mod epoll;
pub mod tcp;

pub type Available = u64;

pub const NONE: u64 = 0x0;
pub const READ: u64 = 0x1;
pub const WRITE: u64 = 0x2;
pub const HUP: u64 = 0x4;
