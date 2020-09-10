extern crate core;
extern crate libc;
extern crate tracing;
extern crate tracing_appender;
extern crate tracing_subscriber;

use std::env;
use libc::{c_void,c_char};
// use tracing::instrument;
use tracing::Level;
use tracing::dispatcher::Dispatch;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::FmtSubscriber;


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

/* TODO: Using { $($body:tt)* } instead of $body:block) because of rustc bug refered to
   in https://github.com/dtolnay/paste/issues/50 */
#[macro_export]
macro_rules! hook {

    (unsafe fn $real_fn:ident ( $($v:ident : $t:ty),* ) -> $r:ty => $hook_fn:ident { $($body:tt)* }) => {
        paste! {
            #[allow(non_camel_case_types)]
            pub struct [<orig_ $real_fn>] {__private_field: ()}
            #[allow(non_upper_case_globals)]
            static [<orig_ $real_fn>]: [<orig_ $real_fn>] = [<orig_ $real_fn>] {__private_field: ()};
    
            impl [<orig_ $real_fn>] {
                fn get(&self) -> unsafe extern fn ( $($v : $t),* ) -> $r {
                    use ::std::sync::Once;
    
                    static mut REAL: *const u8 = 0 as *const u8;
                    static mut ONCE: Once = Once::new();
    
                    unsafe {
                        ONCE.call_once(|| {
                            REAL = $crate::ld_preload::dlsym_next(concat!(stringify!($real_fn), "\0"));
                        });
                        ::std::mem::transmute(REAL)
                    }
                }
            }

            // #[instrument(skip( $($v),* ))]
            pub unsafe fn $hook_fn ($($v : $t),* ) -> $r {
                if stringify!($real_fn) == "fopen" && !MY_DISPATCH_initialized.with(Cell::get) {
                    $($body)*
                } else {
                    MY_DISPATCH.with(|(tracing, my_dispatch, _guard)| {
                        // println!("tracing: {}", tracing);
                        if *tracing {
                            with_default(&my_dispatch, || {
                                // event!(Level::INFO, "{}()", stringify!($real_fn));
                                $($body)*
                            })
                        } else {
                            $($body)*
                        }
                    })
                }
            }
    
            #[no_mangle]
            pub unsafe extern "C" fn $real_fn ( $($v : $t),* ) -> $r {
                if $crate::initialized() {
                    ::std::panic::catch_unwind(|| $hook_fn ( $($v),* )).ok()
                } else {
                    None
                }.unwrap_or_else(|| [<orig_ $real_fn>].get() ( $($v),* ))
            }

        }
    };

    (unsafe fn $real_fn:ident ( $($v:ident : $t:ty),* ) => $hook_fn:ident { $($body:tt)* }) => {
        $crate::hook! { unsafe fn $real_fn ( $($v : $t),* ) -> () => $hook_fn { $($body)* } }
    };
}

/* TODO: Using { $($body:tt)* } instead of $body:block) because of rustc bug refered to
   in https://github.com/dtolnay/paste/issues/50 */
   #[macro_export]
   macro_rules! shook {
   
       (unsafe fn $real_fn:ident ( $($v:ident : $t:ty),* ) -> $r:ty => ($hook_fn:ident,$syscall:ident) { $($body:tt)* }) => {
           paste! {
               #[allow(non_camel_case_types)]
               pub struct [<orig_ $real_fn>] {__private_field: ()}
               #[allow(non_upper_case_globals)]
               static [<orig_ $real_fn>]: [<orig_ $real_fn>] = [<orig_ $real_fn>] {__private_field: ()};
       
               impl [<orig_ $real_fn>] {
                   fn get(&self) -> unsafe extern fn ( $($v : $t),* ) -> $r {
                       static mut REAL: *const u8 = 0 as *const u8;
                       unsafe {
                           let x=REAL;
                            REAL = $crate::ld_preload::dlsym_next(concat!(stringify!($real_fn), "\0"));
                           ::std::mem::transmute(x)
                       }
                   }
               }
   
               // #[instrument(skip( $($v),* ))]
               pub unsafe fn $hook_fn ($($v : $t),* ) -> $r {
                   if stringify!($real_fn) == "fopen" && !MY_DISPATCH_initialized.with(Cell::get) {
                       $($body)*
                   } else {
                       MY_DISPATCH.with(|(tracing, my_dispatch, _guard)| {
                           // println!("tracing: {}", tracing);
                           if *tracing {
                               with_default(&my_dispatch, || {
                                   // event!(Level::INFO, "{}()", stringify!($real_fn));
                                   $($body)*
                               })
                           } else {
                               $($body)*
                           }
                       })
                   }
               }
       
               #[no_mangle]
               pub unsafe extern "C" fn $real_fn ( $($v : $t),* ) -> $r {
                    static mut recursion: bool = false;       
                    if $crate::initialized() {
                        if ([<orig_ $real_fn>].get() as *const u8) != (0 as *const u8) {
                            ::std::panic::catch_unwind(|| $hook_fn ( $($v),* )).ok()
                        } else {
                            ::std::panic::catch_unwind(|| libc::syscall($syscall, $($v),* ) as $r).ok()
                        }
                    } else {
                        None
                    }.unwrap_or_else(|| libc::syscall($syscall, $($v),* ) as $r)
               }
   
           }
       };
   
       (unsafe fn $real_fn:ident ( $($v:ident : $t:ty),* ) => $hook_fn:ident { $($body:tt)* }) => {
           $crate::shook! { unsafe fn $real_fn ( $($v : $t),* ) -> () => $hook_fn { $($body)* } }
       };
   }
   

/* TODO: Using { $($body:tt)* } instead of $body:block) because of rustc bug refered to
   in https://github.com/dtolnay/paste/issues/50 */
#[macro_export]
macro_rules! vhook {

    (unsafe fn $real_fn:ident ( $va:ident : $vaty:ty,  $($v:ident : $t:ty),* ) -> $r:ty => $hook_fn:ident { $($body:tt)* }) => {
        paste! {
            #[allow(non_camel_case_types)]
            pub struct [<orig_ $real_fn>] {__private_field: ()}
            #[allow(non_upper_case_globals)]
            static [<orig_ $real_fn>]: [<orig_ $real_fn>] = [<orig_ $real_fn>] {__private_field: ()};
    
            impl [<orig_ $real_fn>] {
                fn get(&self) -> unsafe extern fn ( $($v : $t),* , $va : $vaty) -> $r {
                    use ::std::sync::Once;
    
                    static mut REAL: *const u8 = 0 as *const u8;
                    static mut ONCE: Once = Once::new();
    
                    unsafe {
                        ONCE.call_once(|| {
                            REAL = $crate::ld_preload::dlsym_next(concat!(stringify!($real_fn), "\0"));
                        });
                        ::std::mem::transmute(REAL)
                    }
                }
            }
    
            // #[instrument(skip( $va, $($v),* ))]
            pub unsafe fn $hook_fn ( $($v : $t),*  , $va : $vaty) -> $r {
                MY_DISPATCH.with(|(tracing, my_dispatch, _guard)| {
                    // println!("tracing: {} {:?}", tracing, $va);
                    if *tracing {
                        with_default(&my_dispatch, || {
                            // event!(Level::INFO, "{}()", stringify!($real_fn));
                            $($body)*
                        })
                    } else {
                        $($body)*
                    }
                })
            }

            #[no_mangle]
            pub unsafe extern "C" fn $real_fn ( $($v : $t),*  , $va : $vaty) -> $r {
                let mut ap: std::ffi::VaListImpl;
                ap = $va.clone();
                if $crate::initialized() {
                    ::std::panic::catch_unwind(|| {
                        let mut aq: std::ffi::VaListImpl;
                        aq = $va.clone();
                        $hook_fn ( $($v),* , aq.as_va_list())
                    }).ok()
                } else {
                    None
                }.unwrap_or_else(|| [<orig_ $real_fn>].get() ( $($v),*  , ap.as_va_list()))
            }
        }

    };

    (unsafe fn $real_fn:ident ( $($v:ident : $t:ty),* ) => $hook_fn:ident { $($body:tt)* }) => {
        $crate::hook! { unsafe fn $real_fn ( $($v : $t),* ) -> () => $hook_fn { $($body)* } }
    };
}

/* TODO: Using { $($body:tt)* } instead of $body:block) because of rustc bug refered to
   in https://github.com/dtolnay/paste/issues/50 */
#[macro_export]
macro_rules! dhook {

    // The orig_hook is needed because variadic functions cannot be associated functions for a structure
    // There is a bug open on this against rust and is being fixed.
    // Until then we need to store the real function pointer in a separately named structure
    (unsafe fn $real_fn:ident ( $va:ident : $vaty:ty,  $($v:ident : $t:ty),* ) -> $r:ty => $hook_fn:ident  { $($body:tt)* }) => {
        paste! {
            #[allow(non_camel_case_types)]
            pub struct [<orig_ $real_fn>] {__private_field: ()}
            #[allow(non_upper_case_globals)]
            static [<orig_ $real_fn>]: [<orig_ $real_fn>] = [<orig_ $real_fn>] {__private_field: ()};
    
            impl [<orig_ $real_fn>] {
                fn get(&self) -> unsafe extern "C" fn ( $($v : $t),* , $va : ...) -> $r {
                    use ::std::sync::Once;
    
                    static mut REAL: *const u8 = 0 as *const u8;
                    static mut ONCE: Once = Once::new();
    
                    unsafe {
                        ONCE.call_once(|| {
                            REAL = $crate::ld_preload::dlsym_next(concat!(stringify!($real_fn), "\0"));
                        });
                        ::std::mem::transmute(REAL)
                    }
                }
            }

            #[no_mangle]
            pub unsafe extern "C" fn $real_fn ( $($v : $t),*  , $va: ...) -> $r {
                ::std::panic::catch_unwind(|| {
                    let mut aq: std::ffi::VaListImpl;
                    aq = $va.clone();
                    $hook_fn ( $($v),* , aq )
                }).unwrap()
            }

        }

            // #[instrument(skip( $va, $($v),* ))]
            pub unsafe fn $hook_fn ( $($v : $t),* , $va: $vaty) -> $r {
                MY_DISPATCH.with(|(tracing, my_dispatch, _guard)| {
                    // println!("tracing: {}, {:?}", tracing, $va);
                    if *tracing {
                        with_default(&my_dispatch, || {
                            // event!(Level::INFO, "{}()", stringify!($real_fn));
                            $($body)*
                        })    
                    } else {
                        $($body)*
                    }
                })
            }    
    };

    (unsafe fn $real_fn:ident ( $va:ident : $vaty:ty,  $($v:ident : $t:ty),* ) => $hook_fn:ident { $($body:tt)* }) => {
        $crate::dhook! { unsafe fn $real_fn ( $va: $vaty, $($v : $t),* ) -> () => $hook_fn { $($body)* } }
    };

}

#[macro_export]
macro_rules! real {
    ($real_fn:ident) => {
        paste! {
            [<orig_ $real_fn>].get()
        }
    };
}
