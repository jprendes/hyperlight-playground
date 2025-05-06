pub use hl_guest_async::{block_on, channel, io, notify, spawn, time, JoinHandle};

mod host_impl {
    use alloc::vec::Vec;
    use hyperlight_guest::error::HyperlightGuestError;
    use core::time::Duration;

    use crate::host_function;

    #[host_function("GetTime")]
    pub fn get_time() -> u64;

    #[host_function("TryInput")]
    pub fn try_read(count: u64) -> Result<Vec<u8>, HyperlightGuestError>;

    #[host_function("PollInput")]
    pub fn poll_read(timeout: u64) -> Result<bool, HyperlightGuestError>;

    #[host_function("Sleep")]
    pub fn sleep(duration: u64) -> Result<(), HyperlightGuestError>;

    #[repr(C)]
    #[allow(non_camel_case_types)]
    struct __timespec {
        tv_sec: i64,
        tv_nsec: i64,
    }

    #[no_mangle]
    extern "C" fn __unixtime() -> __timespec {
        let ts = Duration::from_micros(get_time());
        __timespec {
            tv_sec: ts.as_secs() as _,
            tv_nsec: ts.subsec_nanos() as _,
        }
    }

    #[no_mangle]
    extern "C" fn __try_read(fd: i32, buffer: *mut u8, len: usize) -> i32 {
        if fd != 0 {
            return -1;
        }
        let buffer = unsafe { core::slice::from_raw_parts_mut(buffer, len) };
        let Ok(data) = try_read(len as u64) else {
            return -1
        };
        let n = data.len().min(buffer.len());
        if n == 0 {
            return -2;
        }
        buffer[0..n].copy_from_slice(&data[0..n]);
        if n > buffer.len() {
            return -1;
        }
        return n as _;
    }

    #[no_mangle]
    extern "C" fn __poll_read(fds: *mut i32, nfds: usize, timeout: __timespec) -> i32 {
        let fds = unsafe { core::slice::from_raw_parts_mut(fds, nfds) };
        for fd in fds.iter_mut() {
            if *fd != 0 {
                return -1;
            }
        }
        let timeout = Duration::new(timeout.tv_sec as _, timeout.tv_nsec as _);
        let timeout = timeout.as_micros().min(u64::MAX as _) as u64;
        let Ok(ready) = poll_read(timeout) else {
            return -1
        };
        if !ready {
            for fd in fds.iter_mut() {
                *fd = -1;
            }
            0
        } else {
            fds.len() as _
        }
    }

    #[no_mangle]
    extern "C" fn __sleep(timeout: __timespec) {
        let timeout = Duration::new(timeout.tv_sec as _, timeout.tv_nsec as _);
        let timeout = timeout.as_micros().min(u64::MAX as _) as u64;
        let _ = sleep(timeout);
    }
}
