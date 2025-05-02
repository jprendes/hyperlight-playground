// this library is heavyly inspired by the mini tokio tutorial
//  * https://tokio.rs/tokio/tutorial/async#mini-tokio

#![no_std]

extern crate alloc;

pub mod channel;
mod host;
pub mod io;
pub mod notify;
mod runtime;
pub mod time;

use core::future::Future;

pub use runtime::JoinHandle;

pub fn block_on<T: Send + 'static>(future: impl Future<Output = T> + Send + 'static) -> T {
    runtime::Runtime::global().block_on(future)
}

pub fn spawn<T: Send + 'static>(future: impl Future<Output = T> + Send + 'static) -> JoinHandle<T> {
    runtime::Runtime::global().spawn(future)
}
