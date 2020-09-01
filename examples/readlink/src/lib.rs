extern crate core;
extern crate libc;
#[macro_use]
extern crate ctor; 


use libc::{c_void,c_char,c_int,size_t,ssize_t};

use std::sync::atomic;

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

// extern "C" fn initialize() {
//     Box::new(0u8);
//     INIT_STATE.store(true, atomic::Ordering::SeqCst);
// }

// /* Rust doesn't directly expose __attribute__((constructor)), but this
//  * is how GNU implements it. */
//  #[link_section = ".init_array"]
//  pub static INITIALIZE_CTOR: extern "C" fn() = ::initialize;

#[ctor]
fn initialize() {
    Box::new(0u8);
    INIT_STATE.store(true, atomic::Ordering::SeqCst);
    println!("Constructor");
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


#[allow(non_camel_case_types)]
pub struct readlink {__private_field: ()}
#[allow(non_upper_case_globals)]
static readlink: readlink = readlink {__private_field: ()};

impl readlink {
    fn get(&self) -> unsafe extern fn (path: *const c_char, buf: *mut c_char, bufsiz: size_t) -> ssize_t  {
        use ::std::sync::Once;

        static mut REAL: *const u8 = 0 as *const u8;
        static mut ONCE: Once = Once::new();

        unsafe {
            ONCE.call_once(|| {
                REAL = dlsym_next(concat!("readlink", "\0"));
            });
            ::std::mem::transmute(REAL)
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn readlink(path: *const c_char, buf: *mut c_char, bufsiz: size_t) -> ssize_t {
        println!("readlink");
        if initialized() {
            println!("initialized");
            ::std::panic::catch_unwind(|| my_readlink ( path, buf, bufsiz )).ok()
        } else {
            println!("not initialized");
            None
        }.unwrap_or_else(|| readlink.get() ( path, buf, bufsiz ))
    }
}

pub unsafe fn my_readlink(path: *const c_char, buf: *mut c_char, bufsiz: size_t) -> ssize_t {
    println!("my_readlink");
    readlink.get()(path, buf, bufsiz)
}
