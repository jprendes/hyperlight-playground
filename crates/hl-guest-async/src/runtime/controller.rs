use alloc::sync::Arc;
use core::sync::atomic::AtomicBool;
use core::task::{Context, Waker};
use futures::task::noop_waker;
use spin::Mutex;

#[derive(Default, Clone)]
pub(crate) struct WorkController(Arc<WorkControllerInner>);

struct WorkControllerInner {
    waker: Mutex<Waker>,
    cancelled: AtomicBool,
}

impl Default for WorkControllerInner {
    fn default() -> Self {
        Self {
            waker: Mutex::new(noop_waker()),
            cancelled: AtomicBool::new(false),
        }
    }
}

impl WorkController {
    pub fn update_waker(&self, ctx: &Context<'_>) {
        let mut waker = self.0.waker.lock();
        if !waker.will_wake(ctx.waker()) {
            *waker = ctx.waker().clone();
        }
    }

    pub fn wake_by_ref(&self) {
        self.0.waker.lock().wake_by_ref();
    }

    pub fn cancel(&self) {
        self.0
            .cancelled
            .store(true, core::sync::atomic::Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.cancelled.load(core::sync::atomic::Ordering::SeqCst)
    }
}

impl PartialEq for WorkController {
    fn eq(&self, other: &Self) -> bool {
        Arc::as_ptr(&self.0).eq(&Arc::as_ptr(&other.0))
    }
}

impl Eq for WorkController {}

impl Ord for WorkController {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        Arc::as_ptr(&self.0).cmp(&Arc::as_ptr(&other.0))
    }
}

impl PartialOrd for WorkController {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
