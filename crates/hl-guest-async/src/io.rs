use alloc::{string::String, vec::Vec};
use spin::{Lazy, Mutex, MutexGuard};

use crate::host::try_read;
use crate::runtime::Runtime;

#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum Error {
    Again,
    Other,
}

pub type Result<T> = core::result::Result<T, Error>;

pub async fn read(buf: &mut [u8]) -> Result<usize> {
    loop {
        match try_read(0, buf) {
            Ok(n) => return Ok(n),
            Err(Error::Again) => {},
            Err(e) => return Err(e),
        }

        Runtime::global().schedule_io(0).await;
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

    pub async fn read(&self, buf: &mut [u8]) -> Result<usize> {
        self.lock().read(buf).await
    }

    pub async fn read_line<'a>(&'a self, buf: &'a mut String) -> Result<usize> {
        self.lock().read_line(buf).await
    }

    pub async fn read_line_to_string(&self) -> Result<String> {
        self.lock().read_line_to_string().await
    }
}

impl StdinLock<'_> {
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let buffer = &mut self.inner.buffer;
        while buffer.is_empty() {
            buffer.resize(1024, 0);
            let n = read(buffer).await?;
            buffer.truncate(n);
        }

        let n = core::cmp::min(buf.len(), buffer.len());
        let mut tail = buffer.split_off(n);
        core::mem::swap(&mut tail, buffer);
        buf[0..tail.len()].copy_from_slice(&tail);
        Ok(tail.len())
    }

    pub async fn read_line<'a>(&mut self, buf: &'a mut String) -> Result<usize> {
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

    pub async fn read_line_to_string(&mut self) -> Result<String> {
        let mut buf = String::new();
        self.read_line(&mut buf).await?;
        Ok(buf)
    }
}

impl Stdin {}
