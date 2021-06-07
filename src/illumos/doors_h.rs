/* 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2021 Robert D. French
 */

//! Unsafe Declarations for the illumos Doors API
//!
//! This module merely re-exports the subset of the illumos doors api that we need for this
//! project. It makes no attempt at safety or ergonomics. 
//!
//! Check out [revolving-doors] for an introduction to doors.
//!
//! [revolving-doors]: https://github.com/robertdfrench/revolving-door#revolving-doors


#![allow(non_camel_case_types)]
use libc;

pub type door_server_procedure_t = extern "C" fn(
    cookie: *const libc::c_void,
    argp: *const libc::c_char,
    arg_size: libc::size_t,
    dp: *const door_desc_t,
    n_desc: libc::c_uint,
);

extern "C" {
    // Turns a function into a file descriptor.  See DOOR_CREATE(3C)
    pub fn door_create(
        server_procedure: door_server_procedure_t,
        cookie: *const libc::c_void,
        attributes: door_attr_t,
    ) -> libc::c_int;


    // Invokes a function in another process (assuming `d` is a file descriptor for a door which
    // points to a function in another process).  See DOOR_CALL(3C).
    pub fn door_call(d: libc::c_int, params: *const door_arg_t) -> libc::c_int;


    // The inverse of `door_call`. Use this at the end of `server_procedure` in lieu of the
    // traditional `return` statement to transfer control back to the process which originally
    // issued `door_call`. See DOOR_RETURN(3C).
    pub fn door_return(
        data_ptr: *const libc::c_char,
        data_size: libc::size_t,
        desc_ptr: *const door_desc_t,
        num_desc: libc::c_uint,
    ) -> !; // Like EXIT(3C) or EXECVE(2), this function is terminal.
}


// This is your daily driver, right here. `data_ptr` and `data_size` represent the bytes you want
// to send to the server. `rbuf` and `rsize` represent a space you've set aside to store bytes that
// come back from the server. `desc_ptr` and `desc_num` are for passing any file / socket / door
// descriptors you'd like the server to be able to access. It is described in more detail below.
#[repr(C)]
pub struct door_arg_t {
    pub data_ptr: *const libc::c_char,
    pub data_size: libc::size_t,
    pub desc_ptr: *const door_desc_t,
    pub desc_num: libc::c_uint,
    pub rbuf: *const libc::c_char,
    pub rsize: libc::size_t,
}


// For our purposes, this data structure and its constituent parts are mostly opaque *except* that
// it holds any file / socket / door descriptors which we would like to pass between processes.
// Rust does not support nested type declaration like C does, so we define each component
// separately. See /usr/include/sys/doors.h for the original (nested) definition of this type and
// https://github.com/robertdfrench/revolving-door/tree/master/A0_result_parameters for a visual
// guide.
#[repr(C)]
pub struct door_desc_t {
    pub d_attributes: door_attr_t,
    pub d_data: door_desc_t__d_data,
}


// Door behavior options, as specified in the "Description" section of DOOR_CREATE(3C).
pub type door_attr_t = libc::c_uint;
pub const DOOR_REFUSE_DESC: door_attr_t = 0x40; // Disable file descriptor passing.


// This is not a real doors data structure *per se*, but rather the `d_data` component of the
// `door_dest_t` type. It is defined in /usr/include/sys/doors.h.
#[repr(C)]
pub union door_desc_t__d_data {
    pub d_desc: door_desc_t__d_data__d_desc,
    d_resv: [libc::c_int; 5], /* Reserved by illumos for some undocumented reason */
}

// This is the `d_desc` component of the `d_data` union of the `door_desct_t` structure. See its
// definition in /usr/include/sys/doors.h.
#[derive(Copy,Clone)]
#[repr(C)]
pub struct door_desc_t__d_data__d_desc {
    pub d_descriptor: libc::c_int,
    pub d_id: door_id_t
}


// Some kind of door identifier. The doors API handles this for us, we don't really need to worry
// about it. Or at least, if I should be worried about it, I'm in a lot of trouble.
pub type door_id_t = libc::c_ulonglong;

