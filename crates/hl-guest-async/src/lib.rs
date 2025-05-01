// this library is heavyly inspired by the mini tokio tutorial
//  * https://tokio.rs/tokio/tutorial/async#mini-tokio

#![no_std]

extern crate alloc;

pub mod runtime;
pub mod time;
pub mod io;
mod host;

pub use runtime::Runtime;
