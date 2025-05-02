use super::host::get_time;
use crate::notify::Notify;
use core::future::Future;
use core::time::Duration;
use futures::{select_biased, FutureExt};

pub async fn sleep(duration: Duration) {
    let notify = Notify::new();
    let notified = notify.notified();
    let deadline = get_time() + duration;
    crate::runtime::Runtime::global().schedule_timer(deadline, notify);
    notified.await;
}

pub trait Timeout: Future {
    #[allow(async_fn_in_trait)]
    async fn timeout(self, duration: Duration) -> Option<Self::Output>;
}

impl<F: Future> Timeout for F {
    async fn timeout(self, duration: Duration) -> Option<Self::Output> {
        let mut this = core::pin::pin!(self.fuse());
        select_biased! {
            _ = sleep(duration).fuse() => None,
            result = this => Some(result),
        }
    }
}
