# Enjoyable hyperlight guests

This repo provides two macros to make writing hyperlight Rust guests more enjoyable.

## Functions import and export

The two macros `#[host_function]` and `#[guest_function]` let you import host functions and export guest functions respectively.

```rust
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
```

By default the macros use the function verbatim name for regristration with the host. The registation name can be overriten providing a string to the macro, e.g.: `#[host_function("GetTime")]` or `#[guest_function("MeaningOfLife")]`.

The arguments and return types of the functions must be one of the serializable types: `i32`, `u32`, `i64`, `u64`, `f32`, `f64`, `bool`, `String`, `Vec<u8>`. The return type can also be `()`.

Additionally the return type can be a `Result<T, E>`, where `T` is a serializable type and E can be converted to (guest_function) / from (host_function) [`HyperlightGuestError`](https://docs.rs/hyperlight-guest/latest/hyperlight_guest/error/struct.HyperlightGuestError.html). If the return type is not a result, any error will be `unwrap`ed.

Enabling the `async` feature lets you export async guest functions (see the [async section](#async-guest-functions) below for more details).

<details>
<summary>Example using <code>Result</code></summary>

```rust
#![no_std]
#![no_main]

extern crate alloc;
use alloc::string::{String, ToString as _};
use hl_guest::{guest_function, host_function, println};
pub use hl_guest::error::{ErrorCode, HyperlightGuestError};

struct Error;
impl From<HyperlightGuestError> for Error {
    fn from(_: HyperlightGuestError) -> Self {
        Error
    }
}
impl From<Error> for HyperlightGuestError {
    fn from(_: Error) -> Self {
        HyperlightGuestError::new(ErrorCode::GuestError, "error".to_string())
    }
}

#[host_function("GetTime")]
fn get_time() -> Result<u64, Error>;

#[guest_function]
fn life(name: String) -> Result<i32, Error> {
    let time = get_time()? / 1000000;
    println!("My dear {name}, today at unixtime {time} the meaning of life is 42");
    return Ok(42);
}
```
</details>

## Async guest functions

Enabling the `async` feature you can use the `hl_guest::asyncio` module.
```rust
use hl_guest::asyncio::time::sleep;
use core::time::Duration;

#[guest_function]
async fn slow_echo(name: String) -> String {
    sleep(Duration::from_secs(1)).await
    name
}
```

The `asyncio` module provides a minimal set of async functionalities.
* `block_on`: execute async code in a sync context.
* `spawn`: spawn tasks and joint them with the returned `JoinHandle`.
* `io::stdin`: asynchronously read from stdin.
* `channel::channel`: a minimal unbounded async mpsc channel.
* `notify::Notify`: pause execution of a task until we are notified.
* `time::sleep`: pause execution for a fixed amount of time.

# Running the demo

```bash
cargo build -p guest --target=x86_64-unknown-none && cargo run -p host -- target/x86_64-unknown-none/debug/guest
```

```bash
cargo build -p guest-async --target=x86_64-unknown-none && cargo run -p host -- target/x86_64-unknown-none/debug/guest-async
```