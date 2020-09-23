extern crate core;
extern crate libc;
extern crate tracing;
extern crate tracing_appender;
extern crate tracing_subscriber;

use libc::{c_void,c_char};


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


/* TODO: Using { $($body:tt)* } instead of $body:block) because of rustc bug refered to
   in https://github.com/dtolnay/paste/issues/50 */
#[macro_export]
macro_rules! hook {

    (unsafe fn $real_fn:ident ( $($v:ident : $t:ty),* ) -> $r:ty
                       => ($hook_fn:ident, $syscall:expr, $reqforinit:expr) { $($body:tt)* }) => {
        paste! {
            #[allow(non_camel_case_types)]
            pub struct [<orig_ $real_fn>] {__private_field: ()}
            #[allow(non_upper_case_globals)]
            static [<orig_ $real_fn>]: [<orig_ $real_fn>] = [<orig_ $real_fn>] {__private_field: ()};
    
            impl [<orig_ $real_fn>] {
                fn get(&self) -> unsafe extern fn ( $($v : $t),* ) -> $r {
                    static mut REAL: *const u8 = 0 as *const u8;
                    unsafe {
                        if $reqforinit {
                            if REAL.is_null() {
                                REAL = $crate::ld_preload::dlsym_next(concat!(stringify!($real_fn), "\0"));
                            }
                        } else {
                            use ::std::sync::Once;
                            static mut ONCE: Once = Once::new();
                            ONCE.call_once(|| {
                                REAL = $crate::ld_preload::dlsym_next(concat!(stringify!($real_fn), "\0"));
                            });
                        }
                        ::std::mem::transmute(REAL)
                    }
                }
            }

            // #[instrument(skip( $($v),* ))]
            pub unsafe fn $hook_fn ($($v : $t),* ) -> $r {
                $($body)*
            }
    
            #[no_mangle]
            pub unsafe extern "C" fn $real_fn ( $($v : $t),* ) -> $r {
                if $crate::initialized() {
                    ::std::panic::catch_unwind(|| $hook_fn ( $($v),* )).ok()
                } else {
                    None
                }.unwrap_or_else(|| {
                    if $reqforinit && $syscall >= 0  {
                        libc::syscall($syscall, $($v),* ) as $r
                    } else {
                        [<orig_ $real_fn>].get() ( $($v),* )
                    }
                })
            }
        }
    };

    (unsafe fn $real_fn:ident ( $($v:ident : $t:ty),* ) -> $r:ty => ($hook_fn:ident, $syscall:expr) { $($body:tt)* }) => {
        $crate::hook! { unsafe fn $real_fn ( $($v : $t),* ) -> $r => ($hook_fn, $syscall, false) { $($body)* } }
    };

    (unsafe fn $real_fn:ident ( $($v:ident : $t:ty),* ) -> $r:ty => $hook_fn:ident { $($body:tt)* }) => {
        $crate::hook! { unsafe fn $real_fn ( $($v : $t),* ) -> $r => ($hook_fn, -1, false) { $($body)* } }
    };

    (unsafe fn $real_fn:ident ( $($v:ident : $t:ty),* ) => $hook_fn:ident { $($body:tt)* }) => {
        $crate::hook! { unsafe fn $real_fn ( $($v : $t),* ) -> () => $hook_fn { $($body)* } }
    };
}

/* TODO: Using { $($body:tt)* } instead of $body:block) because of rustc bug refered to
   in https://github.com/dtolnay/paste/issues/50 */
#[macro_export]
macro_rules! vhook {

    (unsafe fn $real_fn:ident ( $va:ident : $vaty:ty,  $($v:ident : $t:ty),* ) -> $r:ty
                                        => ($hook_fn:ident, $reqforinit:expr) { $($body:tt)* }) => {
        paste! {
            #[allow(non_camel_case_types)]
            pub struct [<orig_ $real_fn>] {__private_field: ()}
            #[allow(non_upper_case_globals)]
            static [<orig_ $real_fn>]: [<orig_ $real_fn>] = [<orig_ $real_fn>] {__private_field: ()};
    
            impl [<orig_ $real_fn>] {
                fn get(&self) -> unsafe extern fn ( $($v : $t),* , $va : $vaty) -> $r {
                    static mut REAL: *const u8 = 0 as *const u8;
                    unsafe {
                        if $reqforinit {
                            if REAL.is_null() {
                                REAL = $crate::ld_preload::dlsym_next(concat!(stringify!($real_fn), "\0"));
                            }
                        } else {
                            use ::std::sync::Once;
                            static mut ONCE: Once = Once::new();
                            ONCE.call_once(|| {
                                REAL = $crate::ld_preload::dlsym_next(concat!(stringify!($real_fn), "\0"));
                            });
                        }
                        ::std::mem::transmute(REAL)
                    }
                }
            }

            // #[instrument(skip( $va, $($v),* ))]
            pub unsafe fn $hook_fn ( $($v : $t),*  , $va : $vaty) -> $r {
                $($body)*
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

    (unsafe fn $real_fn:ident ( $($v:ident : $t:ty),* ) -> $r:ty => $hook_fn:ident { $($body:tt)* }) => {
        $crate::vhook! { unsafe fn $real_fn ( $($v : $t),* ) -> $r => ($hook_fn,false) { $($body)* } }
    };

    (unsafe fn $real_fn:ident ( $($v:ident : $t:ty),* ) => $hook_fn:ident { $($body:tt)* }) => {
        $crate::vhook! { unsafe fn $real_fn ( $($v : $t),* ) -> () => $hook_fn { $($body)* } }
    };
}

/* TODO: Using { $($body:tt)* } instead of $body:block) because of rustc bug refered to
   in https://github.com/dtolnay/paste/issues/50 */
#[macro_export]
macro_rules! dhook {

    // The orig_hook is needed because variadic functions cannot be associated functions for a structure
    // There is a bug open on this against rust and is being fixed.
    // Until then we need to store the real function pointer in a separately named structure
    (unsafe fn $real_fn:ident ( $va:ident : $vaty:ty,  $($v:ident : $t:ty),* ) -> $r:ty
                      => ($hook_fn:ident, $reqforinit:expr)  { $($body:tt)* }) => {
        paste! {
            #[allow(non_camel_case_types)]
            pub struct [<orig_ $real_fn>] {__private_field: ()}
            #[allow(non_upper_case_globals)]
            static [<orig_ $real_fn>]: [<orig_ $real_fn>] = [<orig_ $real_fn>] {__private_field: ()};
    
            impl [<orig_ $real_fn>] {
                fn get(&self) -> unsafe extern "C" fn ( $($v : $t),* , $va : ...) -> $r {
                    static mut REAL: *const u8 = 0 as *const u8;
                    unsafe {
                        if $reqforinit {
                            if REAL.is_null() {
                                REAL = $crate::ld_preload::dlsym_next(concat!(stringify!($real_fn), "\0"));
                            }
                        } else {
                            use ::std::sync::Once;
                            static mut ONCE: Once = Once::new();
                            ONCE.call_once(|| {
                                REAL = $crate::ld_preload::dlsym_next(concat!(stringify!($real_fn), "\0"));
                            });
                        }
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
                $($body)*
            }    
    };

    (unsafe fn $real_fn:ident ( $va:ident : $vaty:ty,  $($v:ident : $t:ty),* ) -> $r:ty => $hook_fn:ident  { $($body:tt)* }) => {
        $crate::dhook! { unsafe fn $real_fn ( $va: $vaty, $($v : $t),* ) -> $r => ($hook_fn,false) { $($body)* } }
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
