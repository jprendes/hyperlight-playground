use alloc::collections::VecDeque;
use alloc::sync::Arc;

use spin::Mutex;

#[derive(Clone)]
pub struct Sender<T>(Arc<Mutex<VecDeque<T>>>);
pub struct Receiver<T>(Arc<Mutex<VecDeque<T>>>);

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let queue = Arc::new(Mutex::new(VecDeque::new()));
    let sender = Sender(queue.clone());
    let receiver = Receiver(queue);
    (sender, receiver)
}

impl<T> Sender<T> {
    pub fn send(&self, item: T) {
        let mut queue = self.0.lock();
        queue.push_back(item);
    }
}

impl<T> Receiver<T> {
    pub fn try_recv(&self) -> Option<T> {
        let mut queue = self.0.lock();
        queue.pop_front()
    }
}
