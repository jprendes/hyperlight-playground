#![no_std]
#![no_main]

extern crate alloc;
use core::time::Duration;

use alloc::string::{String, ToString as _};

use hl_guest::io::{stdout, Write as _};
use hl_guest::{guest_function, println, print};
use hl_guest_async::io::stdin;
use hl_guest_async::spawn;
use hl_guest_async::time::{sleep, Timeout as _};

#[guest_function("Main")]
async fn main(name: String) -> i32 {
    println!("Hello {name}, you have 5s to enter your to be greeted");

    spawn(async {
        println!("5 ...");
        for i in 1..5 {
            sleep(Duration::from_secs(1)).await;
            print!("\x1b[s\x1b[A\x1b[G{} ...\x1b[u", 5 - i);
            let _ = stdout().flush();
        }
    });

    let name = stdin()
        .read_line_to_string()
        .timeout(Duration::from_secs(5))
        .await
        .transpose()
        .unwrap()
        .unwrap_or("anonymous".to_string());

    let name = name.trim();

    println!("Hello {name:?}!");

    println!("Goodbye!");

    return 0;
}
