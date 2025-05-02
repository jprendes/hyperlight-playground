#![no_std]
#![no_main]

extern crate alloc;

#[doc(hidden)]
pub mod __private;

use alloc::vec::Vec;
use hyperlight_common::flatbuffer_wrappers::function_call::FunctionCall;

use __private::GUEST_FUNCTION_INIT;

pub use hl_guest_macros::{guest_function, host_function};

pub mod error {
    pub use hyperlight_guest::error::HyperlightGuestError;
    pub use hyperlight_common::flatbuffer_wrappers::guest_error::ErrorCode;
}

use error::{HyperlightGuestError, ErrorCode};

#[no_mangle]
extern "C" fn hyperlight_main() {
    for registration in GUEST_FUNCTION_INIT {
        registration();
    }
}

#[no_mangle]
fn guest_dispatch_function(
    function_call: FunctionCall,
) -> Result<Vec<u8>, HyperlightGuestError> {
    Err(HyperlightGuestError::new(
        ErrorCode::GuestFunctionNotFound,
        function_call.function_name.clone(),
    ))
}

pub mod io;

#[cfg(feature = "async")]
pub mod asyncio;