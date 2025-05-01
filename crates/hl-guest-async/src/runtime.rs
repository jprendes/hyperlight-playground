use alloc::sync::Arc;
use core::cell::UnsafeCell;
use core::cmp::Reverse;
use core::future::Future;
use spin::Mutex;

mod channel;
pub(crate) mod controller;
mod task;
mod work;

use channel::{channel, Receiver, Sender};
use controller::WorkController;
use task::Task;

pub struct Runtime {
    scheduled: Receiver<Arc<Task>>,
    sender: Sender<Arc<Task>>,
    work: Mutex<work::RuntimeWork>,
}

impl Runtime {
    pub fn run(&self) {
        loop {
            while let Some(task) = self.scheduled.try_recv() {
                task.poll();
            }

            if !self.work.lock().work() {
                break;
            }
        }
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
    pub fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
        Task::spawn(future, &self.sender);
    }

    pub(crate) fn schedule_timer(&self, deadline: u64, controller: WorkController) {
        self.work
            .lock()
            .timers
            .push((Reverse(deadline), controller));
    }

    pub(crate) fn schedule_io(&self, controller: WorkController) {
        self.work.lock().ios.push_back(controller);
    }
}
