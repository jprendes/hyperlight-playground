use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use core::time::Duration;

use crate::runtime::controller::WorkController;

use super::host::get_time;

pub async fn sleep(duration: Duration) {
    Sleep::new(duration).await;
}

struct Sleep {
    deadline: u64,
    controller: WorkController,
}

impl Sleep {
    /// Create a new `Delay` future that will complete after the given
    /// duration.
    fn new(duration: Duration) -> Self {
        let deadline = get_time() + duration.as_micros() as u64;
        let controller = WorkController::default();
        super::runtime::Runtime::global().schedule_timer(deadline, controller.clone());
        Self {
            deadline,
            controller,
        }
    }
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let now = get_time();
        if self.deadline <= now {
            return Poll::Ready(());
        }

        self.controller.update_waker(cx);

        Poll::Pending
    }
}

impl Drop for Sleep {
    fn drop(&mut self) {
        self.controller.cancel();
    }
}
