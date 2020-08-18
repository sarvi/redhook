extern crate libc;
extern crate tracing;
extern crate tracing_appender;
extern crate tracing_subscriber;

#[macro_use]
extern crate redhook;

// use tracing::{instrument};
use std::env;
use libc::{size_t,ssize_t,c_char};
use tracing::{Level, event, };
use tracing::dispatcher::{with_default, Dispatch};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::FmtSubscriber;


fn make_dispatch() -> (Dispatch, WorkerGuard) {
    let file_appender;
    if let Ok(tracefile) =  env::var("WISK_TRACEFILE") {
        file_appender = tracing_appender::rolling::never("", tracefile)
    } else {
        file_appender = tracing_appender::rolling::never("", "/dev/null")
    }
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .with_writer(non_blocking)
        .finish();
    (Dispatch::new(subscriber), guard)
}

thread_local!(static MY_DISPATCH: (Dispatch, WorkerGuard) = make_dispatch());

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
