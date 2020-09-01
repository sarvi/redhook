extern crate core;
extern crate libc;
extern crate tracing;
extern crate tracing_appender;
extern crate tracing_subscriber;


use std::env;
use core::cell::Cell;
use libc::{c_void,c_char,c_int,size_t,ssize_t};
use tracing::{Level, event, };
use tracing::dispatcher::{with_default, Dispatch};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::FmtSubscriber;

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

extern "C" fn initialize() {
    Box::new(0u8);
    INIT_STATE.store(true, atomic::Ordering::SeqCst);
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

/* Rust doesn't directly expose __attribute__((constructor)), but this
 * is how GNU implements it. */
#[link_section = ".init_array"]
pub static INITIALIZE_CTOR: extern "C" fn() = ::initialize;

pub fn make_dispatch(tracevar: &str) -> (bool, Dispatch, WorkerGuard) {
    let file_appender;
    let tracing;
    if let Ok(tracefile) =  env::var(tracevar) {
        file_appender = tracing_appender::rolling::never("", tracefile);
        tracing = true
    } else {
        file_appender = tracing_appender::rolling::never("", "/dev/null");
        tracing = false
    }
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .with_writer(non_blocking)
        .finish();
    (tracing, Dispatch::new(subscriber), guard)
}


thread_local! {
    #[allow(nonstandard_style)]
    static MY_DISPATCH_initialized: ::core::cell::Cell<bool> = false.into();
}
thread_local! {
    static MY_DISPATCH: (bool, Dispatch, WorkerGuard) = {
        let ret = make_dispatch("WISK_TRACE");
        // println!("Tracing: {}",ret.0);
        MY_DISPATCH_initialized.with(|it| it.set(true));
        ret
    };
}


#[allow(non_camel_case_types)]
pub struct posix_spawnp {__private_field: ()}
#[allow(non_upper_case_globals)]
static posix_spawnp: posix_spawnp = posix_spawnp {__private_field: ()};

impl posix_spawnp {
    fn get(&self) -> unsafe extern fn (pid: *mut libc::pid_t, file: *const libc::c_char, file_actions: *const libc::posix_spawn_file_actions_t,
        attrp: *const libc::posix_spawnattr_t, argv: *const *const libc::c_char, envp: *const *const libc::c_char) -> libc::c_int  {
        use ::std::sync::Once;

        static mut REAL: *const u8 = 0 as *const u8;
        static mut ONCE: Once = Once::new();

        unsafe {
            ONCE.call_once(|| {
                REAL = dlsym_next(concat!("posix_spawnp", "\0"));
            });
            ::std::mem::transmute(REAL)
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn posix_spawnp(pid: *mut libc::pid_t, file: *const libc::c_char, file_actions: *const libc::posix_spawn_file_actions_t,
        attrp: *const libc::posix_spawnattr_t, argv: *const *const libc::c_char, envp: *const *const libc::c_char) -> libc::c_int {
        if initialized() {
            ::std::panic::catch_unwind(|| my_posix_spawnp ( pid, file, file_actions, attrp, argv, envp )).ok()
        } else {
            None
        }.unwrap_or_else(|| posix_spawnp.get() ( pid, file, file_actions, attrp, argv, envp ))
    }
}

pub unsafe fn my_posix_spawnp(pid: *mut libc::pid_t, file: *const libc::c_char, file_actions: *const libc::posix_spawn_file_actions_t,
    attrp: *const libc::posix_spawnattr_t, argv: *const *const libc::c_char, envp: *const *const libc::c_char) -> libc::c_int {
        MY_DISPATCH.with(|(tracing, my_dispatch, _guard)| {
            with_default(&my_dispatch, || {
                posix_spawnp.get()(pid, file, file_actions, attrp, argv, envp)
            })
        })
}
