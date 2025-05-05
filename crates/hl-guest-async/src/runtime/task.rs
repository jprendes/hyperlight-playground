use alloc::sync::Arc;
use alloc::task::Wake;
use core::future::Future;
use core::task::Context;
use core::task::Poll;
use futures::future::LocalBoxFuture;
use futures::future::{BoxFuture, FutureExt};
use spin::Mutex;

use crate::channel::Sender;

pub struct SpawnTask {
    future: Mutex<BoxFuture<'static, ()>>,
    executor: Sender<Task>,
}

pub struct BlockOnTask {
    executor: Sender<Task>,
}

pub enum Task {
    Spawn(Arc<SpawnTask>),
    BlockOn(Arc<BlockOnTask>),
}

impl Wake for SpawnTask {
    fn wake(self: Arc<Self>) {
        self.executor.send(Task::Spawn(self.clone()));
    }
}

impl Wake for BlockOnTask {
    fn wake(self: Arc<Self>) {
        self.executor.send(Task::BlockOn(self.clone()));
    }
}

impl SpawnTask {
    pub fn poll(self: Arc<Self>) {
        // Create a waker from the `Task` instance. This
        // uses the `Wake` impl from above.
        let waker = self.clone().into();
        let mut cx = Context::from_waker(&waker);

        // No other thread ever tries to lock the future
        let mut future = self.future.try_lock().unwrap();

        // Poll the inner future
        let _ = future.as_mut().poll(&mut cx);
    }
}

impl BlockOnTask {
    pub fn poll<T>(self: Arc<Self>, future: &mut LocalBoxFuture<'_, T>) -> Poll<T> {
        // Create a waker from the `Task` instance. This
        // uses the `Wake` impl from above.
        let waker = self.clone().into();
        let mut cx = Context::from_waker(&waker);

        // Poll the inner future
        future.as_mut().poll(&mut cx)
    }
}

impl Task {
    // Spawns a new task with the given future.
    //
    // Initializes a new Task harness containing the given future and pushes it
    // onto `sender`. The receiver half of the channel will get the task and
    // execute it.
    pub fn spawn(future: impl Future<Output = ()> + Send + 'static, sender: &Sender<Task>) {
        let task = Task::Spawn(Arc::new(SpawnTask {
            future: Mutex::new(future.fuse().boxed()),
            executor: sender.clone(),
        }));
        sender.send(task);
    }

    pub fn block_on(sender: &Sender<Task>) {
        let task = Arc::new(BlockOnTask { executor: sender.clone() });
        sender.send(Task::BlockOn(task.clone()));
    }
}
