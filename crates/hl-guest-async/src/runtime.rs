use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use core::time::Duration;
use futures::future::BoxFuture;
use futures::{select_biased, FutureExt};
use spin::{Mutex, Once};

mod task;
mod work;

use crate::{
    channel::{channel, Receiver, Sender},
    notify::Notify,
};
use task::Task;

pub struct Runtime {
    scheduled: Receiver<Task>,
    sender: Sender<Task>,
    work: Mutex<work::RuntimeWork>,
}

impl Runtime {
    fn new() -> Runtime {
        let (sender, scheduled) = channel();
        Runtime {
            scheduled,
            sender,
            work: Mutex::default(),
        }
    }

    pub fn block_on<T>(&self, future: impl Future<Output = T>) -> T {
        let mut future = future.fuse().boxed_local();
        Task::block_on(&self.sender);
        loop {
            loop {
                let Some(task) = self.scheduled.try_recv() else {
                    break;
                };
                match task {
                    Task::Spawn(t) => t.poll(),
                    Task::BlockOn(t) => {
                        if let Poll::Ready(val) = t.poll(&mut future) {
                            return val;
                        }
                    }
                };
            }
            let mut work = self.work.lock();
            work.work();
        }
    }

    pub fn global() -> &'static Runtime {
        static INSTANCE: Once<Runtime> = Once::new();
        INSTANCE.call_once(|| Runtime::new())
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

    pub(crate) fn schedule_timer(&self, deadline: Duration, notify: Notify) {
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
