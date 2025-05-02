pub use hl_guest_async::{block_on, channel, io, notify, spawn, time, JoinHandle};

mod host_impl {
    use alloc::vec::Vec;
    use core::time::Duration;

    use hl_guest_async::host;
    use linkme::distributed_slice;

    use crate::host_function;

    #[host_function("GetTime")]
    pub fn get_time() -> u64;

    #[host_function("TryInput")]
    pub fn try_read(count: u64) -> Vec<u8>;

    #[host_function("PollInput")]
    pub fn poll_read(timeout: u64) -> bool;

    #[host_function("Sleep")]
    pub fn sleep(duration: u64) -> ();

    struct HyperlightHostFunctions;

    impl host::HostFunctions for HyperlightHostFunctions {
        fn get_time(&self) -> Duration {
            Duration::from_micros(get_time())
        }

        fn try_read(&self, buf: &mut [u8]) -> usize {
            let data = try_read(buf.len() as u64);
            buf[0..data.len()].copy_from_slice(&data);
            data.len()
        }

        fn poll_read(&self, timeout: Duration) -> bool {
            poll_read(timeout.as_micros() as u64)
        }

        fn sleep(&self, duration: Duration) {
            sleep(duration.as_micros() as u64);
        }
    }

    static HL_HOST_FUNCTIONS: HyperlightHostFunctions = HyperlightHostFunctions;

    #[distributed_slice(host::ASYNC_HOST_FUNCTIONS)]
    static HL_ASYNC_HOST_FUNCTIONS: fn() -> &'static dyn host::HostFunctions =
        || &HL_HOST_FUNCTIONS;
}
