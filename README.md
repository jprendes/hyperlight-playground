# Enjoyable hyperlight guests

This repo provides two macros to make writing hyperlight Rust guests more enjoyable.

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

The `#[host_function]` macro lets you import a function that's provided by the host.

The `#[guest_function]` macro lets you export a guest function to the guest.

By default the macros use the function verbatim name for regristration with the host. The registation name can be overriten providing a string to the macro, e.g.: `#[host_function("GetTime")]` or `#[guest_function("MeaningOfLife")]`.

The arguments and return types of the functions must be one of the serializable types: `i32`, `u32`, `i64`, `u64`, `f32`, `f64`, `bool`, `String`, `Vec<u8>`. The return type can also be `()`.

Additionally the return type can be a `Result<T, E>`, where `T` is a serializable type and E can be converted to (guest_function) / from (host_function) [`HyperlightGuestError`](https://docs.rs/hyperlight-guest/latest/hyperlight_guest/error/struct.HyperlightGuestError.html). If the return type is not a result, any error will be `unwrap`ed.

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

# Running the demo

```bash
cargo build -p guest --target=x86_64-unknown-none && cargo run -p host -- target/x86_64-unknown-none/debug/guest
```