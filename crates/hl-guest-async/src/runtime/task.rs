use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::task::Wake;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use spin::Mutex;

use super::channel::Sender;

/// A structure holding a future and the result of
/// the latest call to its `poll` method.
pub(super) struct TaskFuture {
    future: Pin<Box<dyn Future<Output = ()> + Send>>,
    poll: Poll<()>,
}

pub(super) struct Task {
    task_future: Mutex<TaskFuture>,
    executor: Sender<Arc<Task>>,
}

impl Task {
    fn schedule(self: &Arc<Self>) {
        self.executor.send(self.clone());
    }
}

impl Wake for Task {
    fn wake(self: Arc<Self>) {
        self.schedule();
    }
}

impl TaskFuture {
    fn new(future: impl Future<Output = ()> + Send + 'static) -> TaskFuture {
        TaskFuture {
            future: Box::pin(future),
            poll: Poll::Pending,
        }
    }

    fn poll(&mut self, cx: &mut Context<'_>) {
        // Spurious wake-ups are allowed, even after a future has
        // returned `Ready`. However, polling a future which has
        // already returned `Ready` is *not* allowed. For this
        // reason we need to check that the future is still pending
        // before we call it. Failure to do so can lead to a panic.
        if self.poll.is_pending() {
            self.poll = self.future.as_mut().poll(cx);
        }
    }
}

impl Task {
    pub(super) fn poll(self: Arc<Self>) {
        // Create a waker from the `Task` instance. This
        // uses the `Wake` impl from above.
        let waker = self.clone().into();
        let mut cx = Context::from_waker(&waker);

        // No other thread ever tries to lock the task_future
        let mut task_future = self.task_future.try_lock().unwrap();

        // Poll the inner future
        task_future.poll(&mut cx);
    }

    // Spawns a new task with the given future.
    //
    // Initializes a new Task harness containing the given future and pushes it
    // onto `sender`. The receiver half of the channel will get the task and
    // execute it.
    pub(super) fn spawn(future: impl Future<Output = ()> + Send + 'static, sender: &Sender<Arc<Task>>) {
        let task = Arc::new(Task {
            task_future: Mutex::new(TaskFuture::new(future)),
            executor: sender.clone(),
        });

        let _ = sender.send(task);
    }
}
