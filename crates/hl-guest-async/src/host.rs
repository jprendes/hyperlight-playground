use alloc::vec::Vec;
use hl_guest::host_function;

#[host_function("GetTime")]
pub fn get_time() -> u64;

#[host_function("TryInput")]
pub fn try_input(count: u64) -> Vec<u8>;

#[host_function("PollInput")]
pub fn poll_input(timeout: u64) -> bool;

#[host_function("Sleep")]
pub fn sleep(duration: u64) -> ();