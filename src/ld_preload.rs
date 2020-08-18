use libc::{c_char, c_void};

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

#[macro_export]
macro_rules! hook {

    (unsafe fn $real_fn:ident ( $($v:ident : $t:ty),* ) -> $r:ty => $hook_fn:ident $body:block) => {
        #[allow(non_camel_case_types)]
        pub struct $real_fn {__private_field: ()}
        #[allow(non_upper_case_globals)]
        static $real_fn: $real_fn = $real_fn {__private_field: ()};

        impl $real_fn {
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

            #[no_mangle]
            pub unsafe extern "C" fn $real_fn ( $($v : $t),* ) -> $r {
                if $crate::initialized() {
                    ::std::panic::catch_unwind(|| $hook_fn ( $($v),* )).ok()
                } else {
                    None
                }.unwrap_or_else(|| $real_fn.get() ( $($v),* ))
            }
        }

        // #[instrument]
        pub unsafe fn $hook_fn ( $($v : $t),* ) -> $r {
            MY_DISPATCH.with(|(my_dispatch, _guard)| {
                with_default(&my_dispatch, || {
                    event!(Level::INFO, "{}()", stringify!($real_fn));
                    $body
                })
            })
        }
    };

    (unsafe fn $real_fn:ident ( $($v:ident : $t:ty),* ) => $hook_fn:ident $body:block) => {
        $crate::hook! { unsafe fn $real_fn ( $($v : $t),* ) -> () => $hook_fn $body }
    };
}


#[macro_export]
macro_rules! vhook {

    (unsafe fn $real_fn:ident ( $va:ident : $vaty:ty,  $($v:ident : $t:ty),* ) -> $r:ty => $hook_fn:ident $body:block) => {
        #[allow(non_camel_case_types)]
        pub struct $real_fn {__private_field: ()}
        #[allow(non_upper_case_globals)]
        static $real_fn: $real_fn = $real_fn {__private_field: ()};

        impl $real_fn {
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

            #[no_mangle]
            pub unsafe extern "C" fn $real_fn ( $($v : $t),*  , $va : $vaty) -> $r {
                let mut ap: std::ffi::VaListImpl;
                ap = $va.clone();
                if $crate::initialized() {
                    ::std::panic::catch_unwind(|| {
                        let mut aq: std::ffi::VaListImpl;
                        aq = ap.clone();
                        $hook_fn ( $($v),* , aq.as_va_list())
                    }).ok()
                } else {
                    None
                }.unwrap_or_else(|| $real_fn.get() ( $($v),*  , ap.as_va_list()))
            }
        }

        // #[instrument(skip( $va, $($v),* ))]
        pub unsafe fn $hook_fn ( $($v : $t),*  , $va : $vaty) -> $r {
            MY_DISPATCH.with(|(my_dispatch, _guard)| {
                with_default(&my_dispatch, || {
                    event!(Level::INFO, "{}()", stringify!($real_fn));
                    $body
                })
            })
        }
    };

    (unsafe fn $real_fn:ident ( $($v:ident : $t:ty),* ) => $hook_fn:ident $body:block) => {
        $crate::hook! { unsafe fn $real_fn ( $($v : $t),* ) -> () => $hook_fn $body }
    };
}

#[macro_export]
macro_rules! dhook {

    (unsafe fn $real_fn:ident ( $va:ident : $vaty:ty,  $($v:ident : $t:ty),* ) -> $r:ty => $hook_fn:ident $body:block) => {
        #[no_mangle]
        pub unsafe extern "C" fn $real_fn ( $($v : $t),*  , $va: $vaty) -> $r {
            // let mut ap: std::ffi::VaListImpl;
            // ap = $va.clone();
            ::std::panic::catch_unwind(|| {
                let mut aq: std::ffi::VaListImpl;
                aq = $va.clone();
                $hook_fn ( $($v),* , aq )
            }).unwrap()
        }

        // #[instrument(skip( $va, $($v),* ))]
        pub unsafe fn $hook_fn ( $($v : $t),* , $va: $vaty) -> $r {
            MY_DISPATCH.with(|(my_dispatch, _guard)| {
                with_default(&my_dispatch, || {
                    event!(Level::INFO, "{}()", stringify!($real_fn));
                    $body
                })
            })
        }
    };

    (unsafe fn $real_fn:ident ( $va:ident : $vaty:ty,  $($v:ident : $t:ty),* ) => $hook_fn:ident $body:block) => {
        $crate::dhook! { unsafe fn $real_fn ( $va: $vaty, $($v : $t),* ) -> () => $hook_fn $body }
    };
}

#[macro_export]
macro_rules! real {
    ($real_fn:ident) => {
        $real_fn.get()
    };
}
