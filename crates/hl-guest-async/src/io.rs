use alloc::{string::String, vec::Vec};
use spin::{Lazy, Mutex, MutexGuard};

use crate::host::try_read;
use crate::notify::Notify;

pub async fn read(buf: &mut [u8]) -> usize {
    loop {
        let n = try_read(buf);
        if n > 0 {
            return n;
        }

        let notify = Notify::new();
        let notified = notify.notified();
        crate::runtime::Runtime::global().schedule_io(notify);
        notified.await;
    }
}

pub struct Stdin {
    inner: &'static Mutex<StdinInner>,
}

pub struct StdinLock<'a> {
    inner: MutexGuard<'a, StdinInner>,
}

struct StdinInner {
    buffer: Vec<u8>,
}

pub fn stdin() -> Stdin {
    static INNER: Lazy<Mutex<StdinInner>> =
        Lazy::new(|| Mutex::new(StdinInner { buffer: Vec::new() }));
    Stdin { inner: &INNER }
}

#[derive(Clone, Debug)]
pub enum Never {}

impl Stdin {
    pub fn lock(&self) -> StdinLock {
        StdinLock {
            inner: self.inner.lock(),
        }
    }

    pub async fn read(&self, buf: &mut [u8]) -> Result<usize, Never> {
        self.lock().read(buf).await
    }

    pub async fn read_line<'a>(&'a self, buf: &'a mut String) -> Result<usize, Never> {
        self.lock().read_line(buf).await
    }

    pub async fn read_line_to_string(&self) -> Result<String, Never> {
        self.lock().read_line_to_string().await
    }
}

impl StdinLock<'_> {
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Never> {
        let buffer = &mut self.inner.buffer;
        while buffer.is_empty() {
            buffer.resize(1024, 0);
            let n = read(buffer).await;
            buffer.truncate(n);
        }

        let n = core::cmp::min(buf.len(), buffer.len());
        let mut tail = buffer.split_off(n);
        core::mem::swap(&mut tail, buffer);
        buf[0..tail.len()].copy_from_slice(&tail);
        Ok(tail.len())
    }

    pub async fn read_line<'a>(&mut self, buf: &'a mut String) -> Result<usize, Never> {
        let mut bytes = alloc::vec![];
        loop {
            let mut c = 0u8;
            // our implementation of read stops at newline
            self.read(core::slice::from_mut(&mut c)).await?;
            bytes.push(c);
            if c == 0 || c == b'\n' {
                break;
            }
        }
        let bytes = String::from_utf8_lossy(&bytes);
        buf.push_str(&bytes);
        Ok(bytes.len())
    }

    pub async fn read_line_to_string(&mut self) -> Result<String, Never> {
        let mut buf = String::new();
        let _ = self.read_line(&mut buf).await;
        Ok(buf)
    }
}

impl Stdin {}
