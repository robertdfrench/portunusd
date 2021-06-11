/* 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2021 Robert D. French
 */
//! Application Entrypoint for Network Requests.
//!
//! See [revolving-doors][1] for an example, and [`DOOR_CREATE(3C)`] for the original definition.
//!
//! [1]: https://github.com/robertdfrench/revolving-door/blob/master/40_knock_knock/server.c#L8
//! [`DOOR_CREATE(3C)`]: https://illumos.org/man/3c/door_create


use crate::illumos::door_h::{
    door_desc_t,
    door_return
};
use libc;
use std::slice;
use std::ptr;


/// Trait for types derived from the `define_server_procedure!` macro.
///
/// Because `define_server_procedure!` creates a new type to "host" each server procedure, we need
/// to define a trait so we can work with these types generically.
pub trait ServerProcedure {

    /// This is the part you define.  The function body you give in `define_server_procedure!` will
    /// end up as the definition of this `rust` function, which will be called by the associated
    /// `c` function in this trait.
    fn rust(request: &[u8]) -> Vec<u8>;

    /// This is a wrapper that fits the Doors API All it does is pack and unpack data so that our
    /// server procedure doesn't have to deal with the doors api directly.
    extern "C" fn c(
        _cookie: *const libc::c_void,
        argp: *const libc::c_char,
        arg_size: libc::size_t,
        _dp: *const door_desc_t,
        _n_desc: libc::c_uint
    ) {
        let request = unsafe{ slice::from_raw_parts(argp as *const u8, arg_size) };
        let response = Self::rust(request);
        let data_ptr = response.as_ptr();
        let data_size = response.len();
        println!("Should return {} bytes", data_size);
        unsafe{ door_return(data_ptr as *const libc::c_char, data_size, ptr::null(), 0); }
    }
}


/// Create a function that can respond to door invocations!
///
/// Your function will need to accept a slice of `u8` and return `Vec<u8>`. As an example, consider
/// the server procedure below which greets a telnet user:
///
/// ```
/// use portunus::define_server_procedure;
/// use std::fmt::format;
/// use std::str::from_utf8;
///
/// define_server_procedure!(Hello(request: &[u8]) -> Vec<u8> {
///     match from_utf8(request) {
///         Err(_) => vec![],
///         Ok(name) => {
///             let response = format!("Hello, {}!", name);
///             response.into_bytes()
///         }
///     }
/// });
/// ```
///
///
#[macro_export]
macro_rules! define_server_procedure {
    ($i:ident($a:ident: &[u8]) -> Vec<u8> $b:block) => {
        use portunus::server_procedure::ServerProcedure;
        struct $i;
        impl ServerProcedure for $i {
            fn rust($a: &[u8]) -> Vec<u8> $b
        }
    }
}
