#![no_std]
#![no_main]

extern crate alloc;
use core::time::Duration;

use alloc::string::String;
use futures::{select_biased, FutureExt};

use hl_guest::{guest_function, println};
use hl_guest_async::io::stdin;
use hl_guest_async::time::sleep;
use hl_guest_async::Runtime;

#[guest_function("Main")]
fn main(name: String) -> i32 {
    let rt = Runtime::global();

    for i in 1..5 {
        rt.spawn(async move {
            sleep(Duration::from_secs(i)).await;
            println!("{}", 5 - i);
        });
    }

    rt.spawn(async move {
        println!("Hello {name}, you have 5s to enter your name and press enter to be greeted");
        let mut name = String::new();
        let stdin = stdin();
        let mut stdin = stdin.lock();
        let timeout = sleep(Duration::from_secs(5));
        let read = stdin.read_line(&mut name);
        select_biased! {
            _ = timeout.fuse() => {
                println!("Timeout entering your name!");
                return;
            }
            _ = read.fuse() => {
                let name = name.trim();
                println!("Hello, {name}!");
                return;
            }
        };
    });

    rt.run();

    return 0;
}
