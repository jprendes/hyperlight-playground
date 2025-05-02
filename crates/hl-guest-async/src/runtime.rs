use alloc::sync::Arc;
use core::cell::UnsafeCell;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::{Context, Poll};
use futures::future::BoxFuture;
use futures::{select_biased, FutureExt};
use spin::Mutex;

mod task;
mod work;

use crate::{
    channel::{channel, Receiver, Sender},
    notify::Notify,
};
use task::Task;

pub struct Runtime {
    scheduled: Receiver<Arc<Task>>,
    sender: Sender<Arc<Task>>,
    work: Mutex<work::RuntimeWork>,
}

impl Runtime {
    fn run(&self, stop: Arc<AtomicBool>) {
        loop {
            loop {
                if stop.load(Ordering::SeqCst) {
                    return;
                }
                let Some(task) = self.scheduled.try_recv() else {
                    break;
                };
                task.poll();
            }
            if stop.load(Ordering::SeqCst) {
                return;
            }
            let mut work = self.work.lock();
            if !work.work_pending() {
                break;
            }
            work.work();
        }
    }

    pub fn block_on<T: Send + 'static>(
        &self,
        future: impl Future<Output = T> + Send + 'static,
    ) -> T {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_clone = stop.clone();
        let (tx, rx) = channel();
        self.spawn(async move {
            tx.send(future.await);
            stop_clone.store(true, Ordering::SeqCst);
        });
        self.run(stop);
        rx.try_recv().unwrap()
    }

    fn new() -> Runtime {
        let (sender, scheduled) = channel();
        Runtime {
            scheduled,
            sender,
            work: Mutex::default(),
        }
    }

    pub fn global() -> &'static Runtime {
        static INSTANCE: Mutex<Option<UnsafeCell<Runtime>>> = Mutex::new(None);
        let ptr = INSTANCE
            .lock()
            .get_or_insert_with(|| UnsafeCell::new(Runtime::new()))
            .get();
        unsafe { &*ptr }
    }

    /// Spawn a future onto the runtime.
    ///
    /// The given future is wrapped with the `Task` harness and pushed into the
    /// `scheduled` queue. The future will be executed when `run` is called.
    pub fn spawn<T: Send + 'static>(
        &self,
        future: impl Future<Output = T> + Send + 'static,
    ) -> JoinHandle<T> {
        let (tx, rx) = channel();
        let notify = Notify::new();
        let notified = notify.notified();
        Task::spawn(
            async move {
                let result = select_biased! {
                    _ = notified.fuse() => None,
                    result = future.fuse() => Some(result),
                };
                tx.send(result);
            },
            &self.sender,
        );
        JoinHandle {
            result: async move { rx.recv().await }.boxed(),
            abort: notify,
        }
    }

    pub(crate) fn schedule_timer(&self, deadline: u64, notify: Notify) {
        self.work.lock().schedule_timer(deadline, notify);
    }

    pub(crate) fn schedule_io(&self, notify: Notify) {
        self.work.lock().schedule_io(notify);
    }
}

pub struct JoinHandle<T> {
    result: BoxFuture<'static, Option<T>>,
    abort: Notify,
}

impl JoinHandle<()> {
    pub fn abort(self) {
        self.abort.notify_waiters();
    }
}

impl<T> Future for JoinHandle<T> {
    type Output = Option<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.result.as_mut().poll(cx)
    }
}
