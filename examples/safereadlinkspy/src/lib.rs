extern crate core;
extern crate libc;
extern crate tracing;
extern crate tracing_appender;
extern crate tracing_subscriber;
extern crate paste;

#[macro_use]
extern crate redhook;
#[macro_use]
extern crate ctor;

// use tracing::{instrument};
use core::cell::Cell;
use libc::{SYS_readlink,size_t,ssize_t,c_char};
use paste::paste;
use tracing::{Level, event, };
use tracing::dispatcher::{with_default, Dispatch};
use tracing_appender::non_blocking::WorkerGuard;
use redhook::debugtoerr;


 #[ctor]
 fn initialize() {
     redhook::initialize();
     println!("Constructor");
 }


thread_local! {
    #[allow(nonstandard_style)]
    static MY_DISPATCH_initialized: ::core::cell::Cell<bool> = false.into();
}
thread_local! {
    static MY_DISPATCH: (bool, Dispatch, WorkerGuard) = {
        let ret = redhook::ld_preload::make_dispatch("REDHOOK_TRACE");
        MY_DISPATCH_initialized.with(|it| it.set(true));
        ret
    };
}

shook! {
    unsafe fn readlink(path: *const c_char, buf: *mut c_char, bufsiz: size_t) -> ssize_t => (my_readlink,SYS_readlink) {
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
