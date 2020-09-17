#![feature(c_variadic)]

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

use core::cell::Cell;
use std::ffi::CStr;
use libc::{c_char,c_int,size_t,ssize_t, O_CREAT, SYS_readlink};
use paste::paste;
// use tracing::{instrument};
use tracing::{Level, event, };
use tracing::dispatcher::{with_default, Dispatch};
use tracing_appender::non_blocking::WorkerGuard;
use redhook::debug;

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


 #[ctor]
 fn initialize() {
     redhook::initialize();
     println!("Constructor");
     MY_DISPATCH.with(|(_tracing, _my_dispatch, _guard)| { });
 }



hook! {
    unsafe fn readlink(path: *const c_char, buf: *mut c_char, bufsiz: size_t) -> ssize_t
            => (my_readlink,SYS_readlink, true) {
        event!(Level::INFO, "readlink({})", CStr::from_ptr(path).to_string_lossy());
        println!("readlink({})", CStr::from_ptr(path).to_string_lossy());
        real!(readlink)(path, buf, bufsiz)
    }
}

vhook! {
    unsafe fn vprintf(args: std::ffi::VaList, format: *const c_char ) -> c_int => my_vprintf {
        event!(Level::INFO, "printf({})", CStr::from_ptr(format).to_string_lossy());
        real!(vprintf)(format, args)
    }
}


dhook! {
    unsafe fn printf(args: std::ffi::VaListImpl, format: *const c_char ) -> c_int => my_printf {
        event!(Level::INFO, "printf({})", CStr::from_ptr(format).to_string_lossy());
        let mut aq: std::ffi::VaListImpl;
        aq  =  args.clone();
        my_vprintf(format, aq.as_va_list())
    }
}

dhook! {
    unsafe fn open(args: std::ffi::VaListImpl, pathname: *const c_char, flags: c_int ) -> c_int => my_open {
        event!(Level::INFO, "printf({}, {})", CStr::from_ptr(pathname).to_string_lossy(), flags);
        if (flags & O_CREAT) == O_CREAT {
            let mut ap: std::ffi::VaListImpl = args.clone();
            let mode: c_int = ap.arg::<c_int>();
            println!("open({},{}(CREAT),{})", CStr::from_ptr(pathname).to_string_lossy(), flags, mode);
            real!(open)(pathname, flags, mode)
        } else {
            println!("open({},{})", CStr::from_ptr(pathname).to_string_lossy(), flags);
            real!(open)(pathname, flags)
        }
    }
}

#[cfg(target_arch = "x86_64")]
dhook! {
    unsafe fn open64(args: std::ffi::VaListImpl, pathname: *const c_char, flags: c_int ) -> c_int => (my_open64, true) {
        debug(format_args!("open64() intercepted {}\n", CStr::from_ptr(pathname).to_string_lossy()));
        if (flags & O_CREAT) == O_CREAT {
            let mut ap: std::ffi::VaListImpl = args.clone();
            let mode: c_int = ap.arg::<c_int>();
            event!(Level::INFO, "open64({}, {}, {})", CStr::from_ptr(pathname).to_string_lossy(), flags, mode);
            // TRACKER.reportopen(pathname,flags,mode);
            debug(format_args!("open64() continue\n"));
            real!(open64)(pathname, flags, mode)
        } else {
            event!(Level::INFO, "open64({}, {})", CStr::from_ptr(pathname).to_string_lossy(), flags);
            // TRACKER.reportopen(pathname,flags,0);
            debug(format_args!("open64() continue\n"));
            real!(open64)(pathname, flags)
        }
    }
}

 /* int execv(const char *path, char *const argv[]); */

 hook! {
    unsafe fn execv(path: *const libc::c_char, argv: *const *const libc::c_char) -> libc::c_int => my_execv {
        event!(Level::INFO, "execv({})", CStr::from_ptr(path).to_string_lossy());
        real!(execv)(path, argv)
    }
}

 /* int execvp(const char *file, char *const argv[]); */

hook! {
    unsafe fn execvp(file: *const libc::c_char, argv: *const *const libc::c_char) -> libc::c_int => my_execvp {
        event!(Level::INFO, "execvp({})", CStr::from_ptr(file).to_string_lossy());
        real!(execvp)(file, argv)
    }
}

/* int execvpe(const char *file, char *const argv[], char *const envp[]); */

hook! {
    unsafe fn execvpe(file: *const libc::c_char,
                     argv: *const *const libc::c_char, envp: *const *const libc::c_char) -> libc::c_int => my_execvpe {
        event!(Level::INFO, "execvpe({})", CStr::from_ptr(file).to_string_lossy());
        real!(execvpe)(file, argv, envp)
    }
}


/* int execve(const char *pathname, char *const argv[], char *const envp[]); */

hook! {
    unsafe fn execve(pathname: *const libc::c_char,
                     argv: *const *const libc::c_char, envp: *const *const libc::c_char) -> libc::c_int => my_execve {
        event!(Level::INFO, "execve({})", CStr::from_ptr(pathname).to_string_lossy());
        real!(execve)(pathname, argv, envp)
    }
}


 /* int execveat(int dirfd, const char *pathname, char *const argv[], char *const envp[], int flags); */

hook! {
    unsafe fn execveat(dirfd: libc::c_int, pathname: *const libc::c_char,
                       argv: *const *const libc::c_char, envp: *const *const libc::c_char) -> libc::c_int => my_execveat {
        event!(Level::INFO, "execveat({})", CStr::from_ptr(pathname).to_string_lossy());
        real!(execveat)(dirfd, pathname, argv, envp)
    }
}


 /* int posix_spawn(pid_t *pid, const char *path, const posix_spawn_file_actions_t *file_actions,
                    const posix_spawnattr_t *attrp, char *const argv[], char *const envp[]); */

hook! {
    unsafe fn posix_spawn(pid: *mut libc::pid_t, path: *const libc::c_char, file_actions: *const libc::posix_spawn_file_actions_t,
                           attrp: *const libc::posix_spawnattr_t, argv: *const *const libc::c_char, envp: *const *const libc::c_char) -> libc::c_int => my_posix_spawn {
        event!(Level::INFO, "posix_spawn({})", CStr::from_ptr(path).to_string_lossy());
        real!(posix_spawn)(pid, path, file_actions, attrp, argv, envp)
    }
}

/* int posix_spawnp(pid_t *pid, const char *file, const posix_spawn_file_actions_t *file_actions,
                    const posix_spawnattr_t *attrp, char *const argv[], char * const envp[]); */

hook! {
    unsafe fn posix_spawnp(pid: *mut libc::pid_t, file: *const libc::c_char, file_actions: *const libc::posix_spawn_file_actions_t,
                           attrp: *const libc::posix_spawnattr_t, argv: *const *const libc::c_char, envp: *const *const libc::c_char) -> libc::c_int => my_posix_spawnp {
        event!(Level::INFO, "posix_spawnp({})", CStr::from_ptr(file).to_string_lossy());
        real!(posix_spawnp)(pid, file, file_actions, attrp, argv, envp)
    }
}


/* FILE popen(const char *command, const char *type); */

hook! {
    unsafe fn popen(command: *const libc::c_char, ctype: *const libc::c_char) -> *const libc::FILE => my_popen {
        event!(Level::INFO, "popen({})", CStr::from_ptr(command).to_string_lossy());
        real!(popen)(command, ctype)
    }
}