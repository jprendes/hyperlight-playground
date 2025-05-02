use core::time::Duration;

use linkme::distributed_slice;
use spin::Once;

#[distributed_slice]
pub static ASYNC_HOST_FUNCTIONS: [fn() -> &'static dyn HostFunctions];

pub trait HostFunctions: Send + Sync + 'static {
    fn get_time(&self) -> Duration;
    fn try_read(&self, buf: &mut [u8]) -> usize;
    fn poll_read(&self, timeout: Option<Duration>) -> bool;
    fn sleep(&self, duration: Option<Duration>) -> ();
}

fn get_host_functions() -> &'static dyn HostFunctions {
    static FCN: Once<&'static dyn HostFunctions> = Once::new();
    *FCN.call_once(|| {
        ASYNC_HOST_FUNCTIONS.first().unwrap()()
    })
}

pub fn get_time() -> Duration {
    get_host_functions().get_time()
}

pub fn try_read(buf: &mut [u8]) -> usize {
    get_host_functions().try_read(buf)
}

pub fn poll_read(timeout: Option<Duration>) -> bool {
    get_host_functions().poll_read(timeout)
}

pub fn sleep(duration: Option<Duration>) {
    get_host_functions().sleep(duration)
}
