use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::Waker;

use alloc::collections::VecDeque;
use alloc::sync::Arc;
use core::task::{Context, Poll};
use futures::task::noop_waker;
use spin::Mutex;

#[derive(Default)]
pub struct Notify {
    waiters: Mutex<VecDeque<NotifiedInner>>,
}

pub struct Notified {
    inner: NotifiedInner,
}

#[derive(Clone)]
pub struct NotifiedInner {
    notified: Arc<AtomicBool>,
    cancelled: Arc<AtomicBool>,
    waker: Arc<Mutex<Waker>>,
}

impl Drop for Notified {
    fn drop(&mut self) {
        self.inner.cancelled.store(true, Ordering::SeqCst);
    }
}

impl Notify {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn notify_one(&self) {
        let mut waiters = self.waiters.lock();
        while let Some(waiter) = waiters.pop_front() {
            if !waiter.cancelled.load(Ordering::SeqCst) {
                waiter.notified.store(true, Ordering::SeqCst);
                waiter.waker.lock().wake_by_ref();
                break;
            }
        }
    }

    pub fn notify_last(&self) {
        let mut waiters = self.waiters.lock();
        while let Some(waiter) = waiters.pop_back() {
            if !waiter.cancelled.load(Ordering::SeqCst) {
                waiter.notified.store(true, Ordering::SeqCst);
                waiter.waker.lock().wake_by_ref();
                break;
            }
        }
    }

    pub fn notify_waiters(&self) {
        let mut waiters = self.waiters.lock();
        while let Some(waiter) = waiters.pop_front() {
            if !waiter.cancelled.load(Ordering::SeqCst) {
                waiter.notified.store(true, Ordering::SeqCst);
                waiter.waker.lock().wake_by_ref();
            }
        }
    }

    pub fn notified(&self) -> Notified {
        let inner = NotifiedInner {
            notified: Arc::new(AtomicBool::new(false)),
            cancelled: Arc::new(AtomicBool::new(false)),
            waker: Arc::new(Mutex::new(noop_waker())),
        };
        self.waiters.lock().push_back(inner.clone());
        Notified { inner }
    }
}

impl Future for Notified {
    type Output = ();

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<()> {
        if self.inner.notified.load(Ordering::SeqCst) {
            self.inner.notified.store(false, Ordering::SeqCst);
            return Poll::Ready(());
        }

        let mut waker = self.inner.waker.lock();
        if !waker.will_wake(ctx.waker()) {
            *waker = ctx.waker().clone();
        }

        Poll::Pending
    }
}
