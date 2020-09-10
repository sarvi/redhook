extern crate core;
extern crate libc;
#[macro_use]
extern crate ctor; 


use libc::{c_void,c_char,c_int,size_t,ssize_t};
use std::io::Write;
use std::sync::atomic;
use std::process;
use std::thread;

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

#[ctor]
fn initialize() {
    print(format_args!("Constructor: begin, initialized={}, pid={}, thread_id={:?}\n", initialized(), process::id(),thread::current().id()));
    Box::new(0u8);
    readlink_get();
    INIT_STATE.store(true, atomic::Ordering::SeqCst);
    print(format_args!("Constructor: end, initialized={}, pid={}, thread_id={:?}\n", initialized(), process::id(),thread::current().id()));
}


#[link(name = "dl")]
extern "C" {
    fn dlsym(handle: *const c_void, symbol: *const c_char) -> *const c_void;
}

const RTLD_NEXT: *const c_void = -1isize as *const c_void;

pub unsafe fn dlsym_next(symbol: &'static str) -> *const u8 {
    let ptr = dlsym(RTLD_NEXT, symbol.as_ptr() as *const c_char);
    if ptr.is_null() {
        panic!("redhook: Unable to find underlying function for {}", symbol);
    }
    ptr as *const u8
}

type Readlink = unsafe extern "C" fn (path: *const c_char, buf: *mut c_char, bufsiz: size_t) -> ssize_t;

#[no_mangle]
pub unsafe extern "C" fn readlink(path: *const c_char, buf: *mut c_char, bufsiz: size_t) -> ssize_t {
    let readl:Readlink;
    if !initialized() {
        print(format_args!("readlink: syscall because not initialized\n"));
        libc::syscall(libc::SYS_readlink, path, buf, bufsiz) as ssize_t
    } else {
        readl = readlink_get();
        if (readl as *const u8) == (0 as *const u8) {
            print(format_args!("readlink: syscall\n"));
            libc::syscall(libc::SYS_readlink, path, buf, bufsiz) as ssize_t
        } else {
            print(format_args!("readlink: get\n"));
            readl(path, buf, bufsiz)
        }
    }
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
