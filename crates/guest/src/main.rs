#![no_std]
#![no_main]

extern crate alloc;
use alloc::string::String;
use hl_guest::{guest_function, host_function, println};

#[host_function("GetTime")]
fn get_time() -> u64;

#[guest_function]
fn life(name: String) -> i32 {
    let time = get_time() / 1000000;
    println!("My dear {name}, today at unixtime {time} the meaning of life is 42");
    return 42;
}
