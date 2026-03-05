//! Safe wrappers around libc / unsafe OS primitives.
//!
//! All `unsafe` code outside `pty.rs` is consolidated here so that every
//! other module can `#![forbid(unsafe_code)]`.

#![allow(unsafe_code)]

use std::os::fd::{BorrowedFd, RawFd};

/// Get the current user's UID.
pub fn getuid() -> u32 {
    unsafe { libc::getuid() }
}

/// Send a signal to a process.
pub fn kill(pid: i32, sig: i32) -> std::io::Result<()> {
    let ret = unsafe { libc::kill(pid as libc::pid_t, sig) };
    if ret == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

/// Get the system page size.
pub fn page_size() -> u64 {
    unsafe { libc::sysconf(libc::_SC_PAGESIZE) as u64 }
}

/// Borrow a raw file descriptor.
///
/// # Safety contract
///
/// The caller must ensure `fd` is a valid, open file descriptor for the
/// lifetime of the returned `BorrowedFd`.  This wrapper exists so that
/// call-sites outside this module don't need `unsafe` blocks.
pub fn borrow_fd(fd: RawFd) -> BorrowedFd<'static> {
    // SAFETY: Callers are responsible for ensuring fd validity.
    // The 'static lifetime is technically a lie — callers must ensure
    // the fd outlives any use of the BorrowedFd.
    unsafe { BorrowedFd::borrow_raw(fd) }
}

/// Query the terminal size (rows, cols) of stdout.
pub fn terminal_size() -> Option<(u16, u16)> {
    use libc::{TIOCGWINSZ, winsize};
    use std::os::unix::io::AsRawFd;

    let fd = std::io::stdout().as_raw_fd();
    let mut ws: winsize = unsafe { std::mem::zeroed() };
    let result = unsafe { libc::ioctl(fd, TIOCGWINSZ, &mut ws) };
    if result == 0 && ws.ws_row > 0 && ws.ws_col > 0 {
        Some((ws.ws_row, ws.ws_col))
    } else {
        None
    }
}
