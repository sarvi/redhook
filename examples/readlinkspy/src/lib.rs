extern crate core;
extern crate libc;
extern crate tracing;
extern crate tracing_appender;
extern crate tracing_subscriber;

#[macro_use]
extern crate redhook;
#[macro_use]
extern crate ctor;

// use tracing::{instrument};
use std::sync::atomic;
use core::cell::Cell;
use libc::{size_t,ssize_t,c_char};
use tracing::{Level, event, };
use tracing::dispatcher::{with_default, Dispatch};
use tracing_appender::non_blocking::WorkerGuard;
use redhook::ld_preload::make_dispatch;

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


thread_local! {
    #[allow(nonstandard_style)]
    static MY_DISPATCH_initialized: ::core::cell::Cell<bool> = false.into();
}
thread_local! {
    static MY_DISPATCH: (bool, Dispatch, WorkerGuard) = {
        let ret = make_dispatch("REDHOOK_TRACE");
        MY_DISPATCH_initialized.with(|it| it.set(true));
        ret
    };
}

hook! {
    unsafe fn readlink(path: *const c_char, buf: *mut c_char, bufsiz: size_t) -> ssize_t => my_readlink {
        if let Ok(path) = std::str::from_utf8(std::ffi::CStr::from_ptr(path).to_bytes()) {
            if path == "test-panic" {
                panic!("Testing panics");
            }
            println!("readlink(\"{}\")", path);
        } else {
            println!("readlink(...)");
        }

        real!(readlink)(path, buf, bufsiz)
    }
}
