use core::time::Duration;

use crate::io::{Error, Result};

#[repr(C)]
#[allow(non_camel_case_types)]
struct __timespec {
    tv_sec: i64,
    tv_nsec: i64,
}

extern "C" {
    fn __unixtime() -> __timespec;
    fn __try_read(fd: i32, buffer: *mut u8, len: usize) -> i32;
    fn __poll_read(fds: *mut i32, nfds: usize, timeout: __timespec) -> i32;
    fn __sleep(timeout: __timespec);
}

pub fn get_time() -> Duration {
    let ts = unsafe { __unixtime() };
    Duration::new(ts.tv_sec as _, ts.tv_nsec as _)
}

pub fn try_read(fd: i32, buf: &mut [u8]) -> Result<usize> {
    let ret = unsafe { __try_read(fd, buf.as_mut_ptr(), buf.len()) };
    match ret {
        0.. => Ok(ret as _),
        -2 => Err(Error::Again),
        _ => Err(Error::Other),
    }
}

pub fn poll_read(mut fds: impl AsMut<[i32]>, timeout: Option<Duration>) -> Result<usize> {
    if let Some(Duration::ZERO) = timeout {
        for fd in fds.as_mut() {
            *fd = -1;
        }
        return Ok(0);
    }
    let timeout = timeout.unwrap_or_default();
    let timeout = __timespec {
        tv_sec: timeout.as_secs() as _,
        tv_nsec: timeout.subsec_nanos() as _,
    };
    let fds = fds.as_mut();
    let ret = unsafe { __poll_read(fds.as_mut_ptr(), fds.len(), timeout) };
    if ret < 0 {
        return Err(Error::Other);
    }
    Ok(ret as _)
}

pub fn sleep(duration: Option<Duration>) {
    if let Some(Duration::ZERO) = duration {
        return;
    }
    let duration = duration.unwrap_or_default();
    let duration = __timespec {
        tv_sec: duration.as_secs() as _,
        tv_nsec: duration.subsec_nanos() as _,
    };
    unsafe { __sleep(duration) }
}
