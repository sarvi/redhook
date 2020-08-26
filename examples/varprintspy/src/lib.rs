#![feature(c_variadic)]

extern crate core;
extern crate libc;
extern crate tracing;
extern crate tracing_appender;
extern crate tracing_subscriber;

#[macro_use]
extern crate redhook;

use core::cell::Cell;
use libc::{c_char,c_int,size_t,ssize_t};
// use tracing::{instrument};
use tracing::{Level, event, };
use tracing::dispatcher::{with_default, Dispatch};
use tracing_appender::non_blocking::WorkerGuard;
use redhook::ld_preload::make_dispatch;

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
            println!("readlink('{}') -> Intercepted", path);
        } else {
            println!("readlink(...)-> Intercepted");
        }

        real!(readlink)(path, buf, bufsiz)
    }
}

vhook! {
    unsafe fn vprintf(args: std::ffi::VaList, format: *const c_char ) -> c_int => my_vprintf {
        if let Ok(format) = std::str::from_utf8(std::ffi::CStr::from_ptr(format).to_bytes()) {
            println!("Rust: vprintf('{}') -> Intercepted", format);
        } else {
            println!("Rust: vprintf(...) -> Intercepted");
        }
        real!(vprintf)(format, args)
    }
}


dhook! {
    unsafe fn printf(args: std::ffi::VaListImpl, format: *const c_char ) -> c_int => my_printf {
        let mut aq: std::ffi::VaListImpl;
        aq  =  args.clone();
        if let Ok(format) = std::str::from_utf8(std::ffi::CStr::from_ptr(format).to_bytes()) {
            println!("Rust: dprintf('{}') -> Intercepted", format);
        } else {
            println!("Rust: dprintf(...) -> Intercepted");
        }
        my_vprintf(format, aq.as_va_list())
    }
}

// #[no_mangle]
// pub unsafe extern "C" fn vprintf (_format: *const c_char, mut _args: std::ffi::VaList)  -> c_int {
//     println!("Rust: vprintf -> Intercept");
//     0
// }


// #[no_mangle]
// pub unsafe extern "C" fn printf (format: *const c_char, ...)  -> c_int {
//     let message = std::ffi::CStr::from_ptr(format).to_string_lossy();
//     eprintln!("Rust: {}", message);
//     println!("Rust: printf --> intercept");
//     0
// }
