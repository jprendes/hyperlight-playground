pub extern crate alloc;

pub use linkme;
pub use hyperlight_common;
pub use hyperlight_guest;

pub mod ty;

#[linkme::distributed_slice]
pub static GUEST_FUNCTION_INIT: [fn()];