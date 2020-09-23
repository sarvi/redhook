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

use libc::{SYS_readlink,size_t,ssize_t,c_char};
use paste::paste;


 #[ctor]
 fn initialize() {
    println!("Constructor");
    redhook::initialize();
 }


hook! {
    unsafe fn readlink(path: *const c_char, buf: *mut c_char, bufsiz: size_t) -> ssize_t => (my_readlink,SYS_readlink,true) {
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
