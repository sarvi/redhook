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

    #[no_mangle]
    static environ: *const *const c_char;
}


#[ctor]
fn initialize() {
    print(format_args!("Constructor: begin, initialized={}, pid={}, thread_id={:?}\n", initialized(), process::id(),thread::current().id()));
    Box::new(0u8);
    INIT_STATE.store(true, atomic::Ordering::SeqCst);
    use std::env::VarError::NotPresent;
    if let Some(eval) = env::var_os("LD_PRELOAD") {
        println!("Found {}", eval.into_string().unwrap());
    } else {
        println!("Not found");
    }
    // env::remove_var("LD_PRELOAD");
    // if let Some(eval) = env::var_os("LD_PRELOAD") {
    //     println!("Found {}", eval.into_string().unwrap());
    // } else {
    //     println!("Not found");
    // }
    // assert_eq!(env::var("LD_PRELOAD"), Err(NotPresent));
    let ldp = CString::new("LD_PRELOAD").expect("CString::new failed");
    unsafe {
        unsetenv(ldp.as_ptr());
    }
    if let Some(eval) = env::var_os("LD_PRELOAD") {
        println!("Found {}", eval.into_string().unwrap());
    } else {
        println!("Not found");
    }
    for i in 0 .. {
        unsafe {
            let mut argptr: *const c_char = *(environ.offset(i));
            if argptr != ptr::null() {
                let x= CStr::from_ptr(argptr).to_str().unwrap();
                if x.starts_with("LD_PRELOAD") {
                    println!("environ: {:?}", x);
                    let newstr=CString::new("NOT_LD=somethingelse").unwrap();
                    argptr = newstr.as_ptr();
                    use std::mem;
                    mem::forget(newstr);
                    let tmp= CStr::from_ptr(argptr).to_str().unwrap();
                    println!("nnew environ: {:?}", tmp);
                }
            } else {
                break;
            }    
        }
    }

    assert_eq!(env::var("LD_PRELOAD"), Err(NotPresent));
    print(format_args!("Constructor: end, initialized={}, pid={}, thread_id={:?}\n", initialized(), process::id(),thread::current().id()));
}


