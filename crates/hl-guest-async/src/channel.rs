use alloc::collections::VecDeque;
use alloc::sync::Arc;
use spin::Mutex;

use crate::notify::Notify;

pub struct Sender<T>(Arc<Channel<T>>);
pub struct Receiver<T>(Arc<Channel<T>>);

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Sender(Arc::clone(&self.0))
    }
}

struct Channel<T> {
    data: Mutex<VecDeque<T>>,
    notify: Notify,
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let channel = Arc::new(Channel {
        data: Mutex::default(),
        notify: Notify::default(),
    });
    let sender = Sender(channel.clone());
    let receiver = Receiver(channel);
    (sender, receiver)
}

impl<T> Sender<T> {
    pub fn send(&self, item: T) {
        self.0.data.lock().push_back(item);
        self.0.notify.notify_one();
    }
}

impl<T> Receiver<T> {
    pub fn try_recv(&self) -> Option<T> {
        self.0.data.lock().pop_front()
    }

    pub async fn recv(&self) -> T {
        let mut notified = self.0.notify.notified();
        loop {
            match self.try_recv() {
                Some(item) => return item,
                None => (&mut notified).await,
            }
        }
    }
}
