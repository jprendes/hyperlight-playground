use alloc::{string::String, vec::Vec};
use spin::{Lazy, Mutex, MutexGuard};

mod read;

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

    pub async fn read_line(&self, buf: &mut alloc::string::String) -> Result<usize, Never> {
        self.lock().read_line(buf).await
    }
}

impl StdinLock<'_> {
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Never> {
        let buffer = &mut self.inner.buffer;
        while buffer.is_empty() {
            buffer.resize(1024, 0);
            let n = read::read(buffer).await;
            buffer.truncate(n);
        }

        let n = core::cmp::min(buf.len(), buffer.len());
        let mut tail = buffer.split_off(n);
        core::mem::swap(&mut tail, buffer);
        buf[0..tail.len()].copy_from_slice(&tail);
        Ok(tail.len())
    }

    pub async fn read_line(&mut self, buf: &mut alloc::string::String) -> Result<usize, Never> {
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
}

impl Stdin {}
