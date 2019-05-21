/// Shareable data structures.
pub mod directory;
pub use self::shared_vec::*;
mod shared_vec;
use libc::{self, c_void, close, ftruncate, mmap, munmap, shm_open, shm_unlink};
use std::ffi::CString;
use std::io::Error;
use std::ptr;
use utils::PAGE_SIZE;

struct SharedMemory<T> {
    pub mem: *mut T,
    name: CString,
    size: usize,
}

impl<T> Drop for SharedMemory<T> {
    fn drop(&mut self) {
        unsafe {
            let size = self.size;
            let _ret = munmap(self.mem as *mut c_void, size); // Unmap pages.
                                                              // Record munmap failure.
            let shm_ret = shm_unlink(self.name.as_ptr());
            assert!(shm_ret == 0, "Could not unlink shared memory region");
        }
    }
}

unsafe fn open_shared<T>(name: &str, size: usize) -> SharedMemory<T> {
    // Make sure size is page aligned
    assert!(size & !PAGE_SIZE == 0);
    let name = CString::new(name).unwrap();
    let mut fd = shm_open(
        name.as_ptr(),
        libc::O_CREAT | libc::O_EXCL | libc::O_RDWR,
        0o700,
    );
    if fd == -1 {
        if let Some(e) = Error::last_os_error().raw_os_error() {
            if e == libc::EEXIST {
                shm_unlink(name.as_ptr());
                fd = shm_open(
                    name.as_ptr(),
                    libc::O_CREAT | libc::O_EXCL | libc::O_RDWR,
                    0o700,
                );
            }
        }
    };
    assert!(fd >= 0, "Could not create shared memory segment");
    let ftret = ftruncate(fd, size as i64);
    assert!(ftret == 0, "Could not truncate");
    let address = mmap(
        ptr::null_mut(),
        size,
        libc::PROT_READ | libc::PROT_WRITE,
        libc::MAP_POPULATE | libc::MAP_PRIVATE,
        fd,
        0,
    );
    if address == libc::MAP_FAILED {
        let err_string = CString::new("mmap failed").unwrap();
        libc::perror(err_string.as_ptr());
        panic!("Could not mmap shared region");
    }
    close(fd);
    SharedMemory {
        mem: address as *mut T,
        name,
        size,
    }
}
