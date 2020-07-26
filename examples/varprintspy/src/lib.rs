#![feature(c_variadic)]

extern crate libc;

#[macro_use]
extern crate redhook;

use libc::{c_char,c_int};


vhook! {
    unsafe fn vprintf(args: std::ffi::VaList, format: *const c_char ) -> c_int => my_vprintf {
        if let Ok(format) = std::str::from_utf8(std::ffi::CStr::from_ptr(format).to_bytes()) {
            if format == "test-panic" {
                panic!("Testing panics");
            }
            println!("vprintf(\"{}\")", format);
        } else {
            println!("vprintf(...)");
        }

        real!(vprintf)(format, args)
    }
}


dhook! {
    unsafe fn printf(args: std::ffi::VaListImpl, format: *const c_char ) -> c_int => my_printf {
        let mut aq: std::ffi::VaListImpl;
        aq  =  args.clone();
        if let Ok(format) = std::str::from_utf8(std::ffi::CStr::from_ptr(format).to_bytes()) {
            println!("printf(\"{}\")", format);
        } else {
            println!("printf(...)");
        }
        my_vprintf(format, aq.as_va_list())
    }
}
