/* 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2021 Robert D. French
 */
//! Define connection handlers for your PortunusD Apps!
//!
//! In PortunusD, every incoming connection is forwarded to an external application via
//! [illumos Doors][1]. You can use the `derive_server_procedure!` macro defined in this module to
//! convert a `Fn: &[u8] -> Vec<u8>` function into a PortunusD function handler.
//!
//! Below is an example of an application that accepts a user's name in the request body and
//! returns a polite greeting:
//! ```
//! use doors::derive_server_procedure;
//! use std::fmt::format;
//! use std::str::from_utf8;
//! use std::os::fd::RawFd;
//!
//! // Consider the function `hello`, which returns a polite greeting to a client:
//! fn hello(_: Vec<RawFd>, request: &[u8]) -> (Vec<RawFd>, Vec<u8>) {
//!     match from_utf8(request) {
//!         Err(_) => (vec![], b"I couldn't understand your name!".to_vec()),
//!         Ok(name) => {
//!             let response = format!("Hello, {}!", name);
//!             (vec![], response.into_bytes())
//!         }
//!     }
//! }
//!
//! // We can turn that function into a special type (one that implements ServerProcedure) which
//! // knows how to make the function available via a "door" on the filesystem:
//! derive_server_procedure!(hello as Hello);
//!
//! // make the `hello` function available on the filesystem
//! let hello_server = Hello::install("portunusd_test.04683b").unwrap();
//!
//! // Now a client (even one in another process!) can call this procedure:
//! let hello_client = doors::Client::new("portunusd_test.04683b").unwrap();
//! let _descriptors, greeting = hello_client.call(vec![], b"Portunus").unwrap();
//!
//! assert_eq!(greeting, b"Hello, Portunus!");
//! ```
//!
//! [1]: https://github.com/robertdfrench/revolving-door

pub mod illumos;

use illumos::door_h::{
    door_call,
    door_create,
    door_arg_t,
    DOOR_DESCRIPTOR,
    DOOR_RELEASE,
    door_desc_t,
    door_desc_t__d_data,
    door_desc_t__d_data__d_desc,
    door_return,
};
use illumos::stropts_h::{ fattach, fdetach };
use illumos::errno;
use libc;
use std::ffi;
use std::fmt;
use std::fs::File;
use std::os::fd;
use std::os::fd::AsRawFd;
use std::os::fd::FromRawFd;
use std::os::unix::io::IntoRawFd;
use std::path::Path;
use std::ptr;
use std::slice;


/// A borrowable door client
///
/// Many threads may need to call the same door application. In order to facilitate this, we
/// bend some ownership concepts a little by introducing a derivative type called `ClientRef`. This
/// type has a separate copy of the door descriptor, and can make calls to the door application
/// independently of the `Client` from which it was derived.
///
/// # Caveat
///
/// There is no lifetime association between a `ClientRef` and the `Client` which produced it. If
/// the `Client` is dropped, the door descriptor will be closed, and the `ClientRef` will no longer
/// be able to place door calls.
#[derive(Clone,Copy)]
pub struct ClientRef {
    door_descriptor: libc::c_int
}

impl ClientRef {
    /// Invoke door server procedure.
    ///
    /// This is intended to be called from a dedicated thread. It will block until the server
    /// procedure calls `door_return`. 
    pub fn call(&self, raw_descriptors: Vec<fd::RawFd>, request: &[u8]) -> Result<(Vec<fd::RawFd>,Vec<u8>),Error> {
        let mut response = Vec::with_capacity(1024);
        // the vector has length zero, so rsize is zero, so the overflow handling gets triggered
        // which fucks up alignment so data_ptr > rbuf

        let mut door_descriptors: Vec<door_desc_t> = raw_descriptors.into_iter().map(|raw| {
            unsafe { door_desc_t::from_raw_fd(raw) }
        }).collect();

        let mut arg = door_arg_t {
            data_ptr: request.as_ptr() as *const i8,
            data_size: request.len(),
            desc_ptr: door_descriptors.as_mut_ptr(),
            desc_num: door_descriptors.len() as u32,
            rbuf: response.as_mut_ptr() as *const i8,
            rsize: response.len()
        };

        if unsafe{ door_call(self.door_descriptor, &mut arg) } == -1 {
            return Err(Error::DoorCall(errno()));
        }

        unsafe{ response.set_len(arg.data_size); }

        let slice = unsafe{ std::slice::from_raw_parts(arg.data_ptr as *const u8, arg.data_size) };
        Ok((vec![],slice.to_vec()))
    }
}

/// A Client handle for a door. Used by PortunusD to call your application.
///
/// When your application wants to receive requests from PortunusD, it must create a [Door] on the
/// filesystem which points to a [`ServerProcedure`] function. This Client type is the reciprocal
/// of a ServerProcedure -- it is PortunusD's way of accessing your application, much like a file
/// handle is a means of accessing the bytes which make up a file.
///
/// [Door]: https://github.com/robertdfrench/revolving-door#revolving-doors
/// [`ServerProcedure`]: trait.ServerProcedure.html
pub struct Client {
    door_descriptor: libc::c_int
}

impl Client {
    /// Try to create a new client, given a filesystem path to a door.
    ///
    /// This may fail if the door does not exist, if the path is not a door, or if some other
    /// terrible thing has happened.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self,Error> {
        let door = File::open(path)?;
        let door_descriptor = door.into_raw_fd();
        Ok(Self{ door_descriptor })
    }

    /// Invoke the Server Procedure defined in a PortunusdD application
    ///
    /// Forwad a slice of bytes through a door to a PortunusD application. If successful, the
    /// resulting `Vec<u8>` will contain the bytes returned from the application's server
    /// procedure.
    pub fn call(&self, descriptors: Vec<fd::RawFd>, request: &[u8]) -> Result<(Vec<fd::RawFd>,Vec<u8>),Error> {
        let cr = self.borrow();
        cr.call(descriptors, request)
    }

    /// A copy of the door descriptor that can be called from another thread
    ///
    /// WARNING: Nothing stops the `Client` from going out of scope without invalidating associated
    /// `ClientRef` objects. Use with caution.
    pub fn borrow(&self) -> ClientRef {
        ClientRef{ door_descriptor: self.door_descriptor }
    }
}

impl Drop for Client {
    /// Close PortunusD's connection the remote application
    fn drop(&mut self) {
        unsafe{ libc::close(self.door_descriptor); }
    }
}


/// A server procedure which has been attached to the filesystem.
pub struct Server {
    jamb_path: ffi::CString,
    door_descriptor: libc::c_int
}


/// Door problems.
///
/// Two things can go wrong with a door -- its path can be invalid, or a system call can fail. If a
/// system call fails, one of this enum's variants will be returned corresponding to the failed
/// system call. It will contain the value of `errno` associated with the failed system call.
#[derive(Debug)]
pub enum Error {
    InvalidPath(ffi::NulError),
    InstallJamb(libc::c_int),
    AttachDoor(libc::c_int),
    OpenDoor(std::io::Error),
    DoorCall(libc::c_int),
    CreateDoor(libc::c_int),
}

impl fmt::Display for Error {
    // Need to look up all these errnos with strerror_r or something
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPath(e) => write!(f, "Door path contained a misplaced NULL: {}", e),
            Self::InstallJamb(errno) => write!(f, "Could not install jamb: {}", errno),
            Self::AttachDoor(errno) => write!(f, "Could not attach door: {}", errno),
            Self::OpenDoor(e) => write!(f, "Could not open door: {}", e),
            Self::DoorCall(errno) => write!(f, "Could not call door: {}", errno),
            Self::CreateDoor(errno) => write!(f, "Could not create door: {}", errno)
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(other: std::io::Error) -> Self {
        Self::OpenDoor(other)
    }
}

impl From<ffi::NulError> for Error {
    fn from(other: ffi::NulError) -> Self {
        Self::InvalidPath(other)
    }
}

impl Server {
    /// Hand the current thread over to the door pool.
    ///
    /// This is useful when an application has finished starting up, and we'd like to put the
    /// "main" thread into the thread pool available to door clients. Only use this if there is no
    /// meaningful work for a "main" thread to be doing when the application is otherwise idle.
    pub fn park(&self) -> ! {
        unsafe{ door_return(ptr::null(), 0, ptr::null(), 0); }
    }
}

impl Drop for Server {
    /// Prevent new client requests
    ///
    /// Removes the door from the filesystem, and closes the door, invaliding any active door
    /// descriptors that client processes may have. This will prevent PortunusD from forwarding
    /// additional requests to your application.
    fn drop(&mut self) {
        // Stop new clients from getting a door descriptor
        unsafe{ fdetach(self.jamb_path.as_ptr()); }
        // Remove jamb from filesystem
        unsafe{ libc::unlink(self.jamb_path.as_ptr()); }
        // Stop existing clients from issuing new door_call()s
        unsafe{ libc::close(self.door_descriptor); }
    }
}

impl AsRawFd for door_desc_t {
    fn as_raw_fd(&self) -> fd::RawFd {
        let d_data = &self.d_data;
        let d_desc = unsafe{ d_data.d_desc };
        let d_descriptor = d_desc.d_descriptor;
        d_descriptor as fd::RawFd
    }
}

impl FromRawFd for door_desc_t {
    unsafe fn from_raw_fd(raw: fd::RawFd) -> Self {
        let d_descriptor = raw as libc::c_int;
        let d_id = 0; // TODO: Confirm "door 0" is appropriate / not wrong for passing sockets
        let d_desc = door_desc_t__d_data__d_desc { d_descriptor, d_id };
        let d_data = door_desc_t__d_data { d_desc };

        let d_attributes = DOOR_DESCRIPTOR | DOOR_RELEASE;
        Self { d_attributes, d_data }
    }
}


/// Trait for types derived from the `define_server_procedure!` macro.
///
/// Because `define_server_procedure!` creates a new type to "host" each server procedure, we need
/// to define a trait so we can work with these types generically.
///
pub trait ServerProcedure {

    /// This is the part you define.  The function body you give in `define_server_procedure!` will
    /// end up as the definition of this `rust` function, which will be called by the associated
    /// `c_wrapper` function in this trait.
    fn rust_wrapper(descriptors: Vec<fd::RawFd>, request: &[u8]) -> (Vec<fd::RawFd>, Vec<u8>);

    /// This is a wrapper that fits the Doors API All it does is pack and unpack data so that our
    /// server procedure doesn't have to deal with the doors api directly. Its unusual signature
    /// comes directly from [`DOOR_CREATE(3C)`].
    ///
    /// [`DOOR_CREATE(3C)`]: https://illumos.org/man/3C/door_create
    extern "C" fn c_wrapper(
        _cookie: *const libc::c_void,
        argp: *const libc::c_char,
        arg_size: libc::size_t,
        dp: *const door_desc_t,
        n_desc: libc::c_uint
    ) {
        let request = unsafe{ slice::from_raw_parts(argp as *const u8, arg_size) };
        let in_door_descriptors = unsafe{ slice::from_raw_parts::<door_desc_t>(dp, n_desc as usize) };
        let in_raw_descriptors: Vec<fd::RawFd> = in_door_descriptors.iter().map(|dd| {
            dd.as_raw_fd()
        }).collect();

        let (out_raw_descriptors, response) = Self::rust_wrapper(in_raw_descriptors, request);

        let out_door_descriptors: Vec<door_desc_t> = out_raw_descriptors.into_iter().map(|raw| {
            unsafe{ door_desc_t::from_raw_fd(raw) }
        }).collect();

        let data_ptr = response.as_ptr();
        let data_size = response.len();
        let desc_ptr = out_door_descriptors.as_ptr();
        let desc_size = out_door_descriptors.len();
        unsafe{ door_return(data_ptr as *const libc::c_char, data_size, desc_ptr, desc_size as libc::c_uint); }
    }

    /// Make this procedure available on the filesystem (as a door).
    fn install(path: &str) -> Result<Server,Error> {
        let jamb_path = ffi::CString::new(path)?;

        // Create door
        let door_descriptor = unsafe{ door_create(Self::c_wrapper, ptr::null(), 0) };
        if door_descriptor == -1 {
            return Err(Error::CreateDoor(errno()));
        }

        // Create jamb
        let create_new = libc::O_RDWR | libc::O_CREAT | libc::O_EXCL;
        match unsafe{ libc::open(jamb_path.as_ptr(), create_new, 0400) } {
            -1 => {
                // Clean up the door, since we aren't going to finish
                unsafe{ libc::close(door_descriptor) }; 
                return Err(Error::InstallJamb(errno()))
            },
            jamb_descriptor => unsafe{ libc::close(jamb_descriptor); }
        }

        // Attach door to jamb
        match unsafe{ fattach(door_descriptor, jamb_path.as_ptr()) } {
            -1 => {
                // Clean up the door and jamb, since we aren't going to finish
                unsafe{ libc::close(door_descriptor) }; 
                unsafe{ libc::unlink(jamb_path.as_ptr()); }
                Err(Error::AttachDoor(errno()))
            },
            _ => Ok(Server{ jamb_path, door_descriptor })
        }
    }
}


/// Define a function which can respond to [`DOOR_CALL(3C)`].
///
/// This macro turns a function into a type which implements the [`ServerProcedure`] trait.
/// The function should accept a `&[u8]` and return a `Vec<u8>`, because the `ServerProcedure`
/// trait will expect that signature.
///
/// # Example
/// ```
/// use doors::derive_server_procedure;
/// use std::fmt::format;
/// use std::str::from_utf8;
/// use std::os::fd::RawFd;
///
/// // Consider this function, which returns a polite greeting to a client:
/// fn hello(_: Vec<RawFd>, request: &[u8]) -> (Vec<RawFd>, Vec<u8>) {
///     match from_utf8(request) {
///         Err(_) => (vec![], b"Your name is not valid utf8".to_vec()),
///         Ok(name) => {
///             let response = format!("Hello, {}!", name);
///             (vec![], response.into_bytes())
///         }
///     }
/// }
///
/// // We can use the `derive_server_procedure!` macro to create a
/// // `ServerProcedure` type called `Hello`:
/// derive_server_procedure!(hello as Hello);
///
/// // We can now create a filesystem object known as a "door" which
/// // will give PortunusD the ability to invoke the `hello` function
/// // (as long as "hello.door" is readable by the `portunus` user):
/// Hello::install("hello.door").unwrap();
/// ```
///
/// [`DOOR_CALL(3C)`]: https://illumos.org/man/3C/door_call
/// [`ServerProcedure`]: door/trait.ServerProcedure.html
#[macro_export]
macro_rules! derive_server_procedure {
    ($function_name:ident as $type_name:ident) => {
        use doors::ServerProcedure;
        struct $type_name;
        impl ServerProcedure for $type_name {
            fn rust_wrapper(in_descriptors: Vec<std::os::fd::RawFd>, request: &[u8]) -> (Vec<std::os::fd::RawFd>, Vec<u8>) {
                $function_name(in_descriptors, request)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_fd_to_door_desc_and_back() {
        let raw: fd::RawFd = 6;
        let dd = unsafe{ door_desc_t::from_raw_fd(raw) };
        assert_eq!(dd.as_raw_fd(), raw);
    }
}
