extern crate core;
extern crate libc;
#[macro_use]
extern crate ctor; 


use libc::{c_void,c_char,c_int,size_t,ssize_t};
use std::ffi::CString;
use std::ffi::CStr;
use std::io::Write;
use std::sync::atomic;
use std::process;
use std::thread;
use std::env;
use std::ptr;


#[cfg(any(target_os = "macos", target_os = "ios"))]
pub mod dyld_insert_libraries;

/* Some Rust library functionality (e.g., jemalloc) initializes
 * lazily, after the hooking library has inserted itself into the call
 * path. If the initialization uses any hooked functions, this will lead
 * to an infinite loop. Work around this by running some initialization
 * code in a static constructor, and bypassing all hooks until it has
 * completed. */

static INIT_STATE: atomic::AtomicBool = atomic::AtomicBool::new(false);

pub fn initialized() -> bool {
    INIT_STATE.load(atomic::Ordering::SeqCst)
}

// This fn works fine when used like this:
// print(format_args!("hello {}", 1));
fn print(args: std::fmt::Arguments<'_>) {
    std::io::stderr().write_fmt(args).unwrap()
}

extern "C" {
    fn unsetenv(string: *const c_char) -> c_int;
}


#[ctor]
fn initialize() {
    print(format_args!("Constructor: begin, initialized={}, pid={}, thread_id={:?}\n", initialized(), process::id(),thread::current().id()));
    Box::new(0u8);
    readlink_get();
    INIT_STATE.store(true, atomic::Ordering::SeqCst);
    print(format_args!("Constructor: end, initialized={}, pid={}, thread_id={:?}\n", initialized(), process::id(),thread::current().id()));
}


fn readlink_get() -> unsafe extern fn (path: *const c_char, buf: *mut c_char, bufsiz: size_t) -> ssize_t  {
    static mut REAL: *const u8 = 0 as *const u8;
    unsafe {
         print(format_args!("readlink_get: REAL: {:?}\n", REAL));
         let x=REAL;
         if (REAL as *const u8) == (0 as *const u8) {
             REAL = dlsym_next(concat!("readlink", "\0"));
             print(format_args!("readlink_get: done dlsym_next: REAL: {:?}\n", REAL));
         }
         ::std::mem::transmute(x)
     }
}