use std::io::{stdin, Read as _};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

pub struct BlockingStdin(Arc<(Mutex<Vec<u8>>, Condvar)>);

impl BlockingStdin {
    pub fn new() -> Self {
        let inner = Arc::new((Mutex::new(Vec::new()), Condvar::new()));
        let inner2 = inner.clone();
        std::thread::spawn(move || {
            let mut stdin = stdin();
            let mut buf = vec![0u8; 1024];
            loop {
                let n = stdin.read(&mut buf).unwrap();
                if n == 0 {
                    break;
                }
                let mut buffer = inner2.0.lock().unwrap();
                buffer.extend_from_slice(&buf[..n]);
                inner2.1.notify_all();
            }
        });
        BlockingStdin(inner)
    }

    pub fn spawn(&self) {
        let inner = self.0.clone();
        std::thread::spawn(move || {
            let mut stdin = stdin();
            let mut buf = vec![0u8; 1024];
            loop {
                let n = stdin.read(&mut buf).unwrap();
                if n == 0 {
                    break;
                }
                let mut buffer = inner.0.lock().unwrap();
                buffer.extend_from_slice(&buf[..n]);
                inner.1.notify_all();
            }
        });
    }

    pub fn try_read(&self, count: usize) -> Vec<u8> {
        let mut buffer = self.0 .0.lock().unwrap();
        if buffer.is_empty() {
            return vec![];
        }
        let count = std::cmp::min(count, buffer.len());
        let mut tail = buffer.split_off(count);
        std::mem::swap(&mut tail, &mut *buffer);
        return tail;
    }

    pub fn poll_data(&self, timeout: Duration) -> bool {
        let buffer = self.0 .0.lock().unwrap();
        let buffer = if timeout.is_zero() {
            self.0 .1.wait_while(buffer, |b| b.is_empty()).unwrap()
        } else {
            self.0
                 .1
                .wait_timeout_while(buffer, timeout, |b| b.is_empty())
                .unwrap()
                .0
        };
        !buffer.is_empty()
    }

    pub fn read(&self, count: usize) -> Vec<u8> {
        let buffer = self.0 .0.lock().unwrap();
        let mut buffer = self.0 .1.wait_while(buffer, |b| b.is_empty()).unwrap();
        if buffer.len() < count {
            return std::mem::take(&mut *buffer);
        }
        let mut tail = buffer.split_off(count);
        std::mem::swap(&mut tail, &mut *buffer);
        return tail;
    }
}
