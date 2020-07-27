extern crate libc;
extern crate tracing;

#[macro_use]
extern crate redhook;

use tracing::{instrument};
use libc::uid_t;

hook! {
    unsafe fn getuid() -> uid_t => i_am_root {
        0
    }
}
