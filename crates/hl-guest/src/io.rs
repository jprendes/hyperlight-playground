extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec;

use core::fmt::Write as _;

use hyperlight_guest::error::HyperlightGuestError;
use spin::Mutex;

use crate::host_function;

#[host_function("HostPrint")]
fn print(msg: String) -> Result<i32, HyperlightGuestError>;

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize, HyperlightGuestError>;
    fn flush(&mut self) -> Result<(), HyperlightGuestError>;
}

static BUFFER: Mutex<String> = Mutex::new(String::new());

pub struct Stdout;

pub fn stdout() -> Stdout {
    Stdout
}

impl Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> Result<usize, HyperlightGuestError> {
        let msg = String::from_utf8_lossy(buf);
        let mut buffer = BUFFER.lock();
        buffer.push_str(&msg);
        if let Some(n) = buffer.rfind('\n') {
            let _ = print(buffer[..=n].to_string())?;
            *buffer = buffer[n + 1..].into();
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), HyperlightGuestError> {
        let mut buffer = BUFFER.lock();
        let _ = print(buffer[..].to_string())?;
        buffer.clear();
        Ok(())
    }
}

impl core::fmt::Write for Stdout {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write(s.as_bytes()).map_err(|_| core::fmt::Error)?;
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    let _ = Stdout.write_fmt(args);
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::io::_print(core::format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n");
    };
    ($($arg:tt)*) => {{
        $crate::print!($($arg)*);
        $crate::print!("\n");
    }};
}
