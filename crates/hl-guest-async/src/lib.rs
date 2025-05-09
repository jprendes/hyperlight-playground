// this library is heavyly inspired by the mini tokio tutorial
//  * https://tokio.rs/tokio/tutorial/async#mini-tokio

#![no_std]

extern crate alloc;

#[doc(hidden)]
mod host;
pub mod channel;
pub mod io;
pub mod notify;
mod runtime;
pub mod time;

use core::future::Future;

pub use runtime::JoinHandle;

pub fn block_on<T>(future: impl Future<Output = T>) -> T {
    runtime::Runtime::global().block_on(future)
}

pub fn spawn<T: Send + 'static>(future: impl Future<Output = T> + Send + 'static) -> JoinHandle<T> {
    runtime::Runtime::global().spawn(future)
}
