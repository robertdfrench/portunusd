/* 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2021 Robert D. French
 */
//! Allow another application to call your function
//!
//! The ApplicationDoorway is all you need.

use crate::path;
use crate::door;
use crate::illumos;
use libc;
use std::ffi;
use std::ptr;


#[derive(Debug)]
pub enum DoorwayError {
    Path(path::Error),
    Door(door::DoorCreationError),
    Fattach(libc::c_int)
}


/// Respond to clients
///
/// ```
/// use portunusd::define_server_procedure;
/// use portunusd::{door,path,illumos,application_doorway};
/// use std::ffi;
/// use std::fmt::format;
/// use std::str::from_utf8;
/// use std::ptr;
/// use libc;
///
/// define_server_procedure!(Greet(name: &[u8]) -> Vec<u8> {
///     match from_utf8(name) {
///         Err(_) => vec![],
///         Ok(name) => {
///             let response = format!("Hello, {}!", name.trim());
///             response.into_bytes()
///         }
///     }
/// });
/// let d = door::Door::create(Greet::c).unwrap();
/// let j = path::Reservation::make("portunusd_b3d839.door").unwrap();
/// let a = path::Attachment::new(d.descriptor,j).unwrap();
///
/// // Pretend to be a client and invoke the Doorway
/// let name = ffi::CString::new("PortunusD").unwrap();
/// unsafe {
///     // Connect to the Capitalization Server through its door.
///     let client_door_fd = libc::open(a.path_ptr(), libc::O_RDONLY);
///
///     // Pass `original` through the Capitalization Server's door.
///     let data_ptr = name.as_ptr();
///     let data_size = name.as_bytes_with_nul().len() - 1; // omit the nul byte
///     let desc_ptr = ptr::null();
///     let desc_num = 0;
///     let rsize = data_size + 64;
///     let rbuf = libc::malloc(rsize) as *mut libc::c_char;
///
///     let params = illumos::door_h::door_arg_t {
///         data_ptr,
///         data_size,
///         desc_ptr,
///         desc_num,
///         rbuf,
///         rsize
///     };
///
///     // This is where the magic happens. We block here while control is transferred to a
///     // separate thread which executes `capitalize_string` on our behalf.
///     illumos::door_h::door_call(client_door_fd, &params);
///
///     // Unpack the returned bytes and compare!
///     let greeting = ffi::CStr::from_ptr(rbuf);
///     let greeting = greeting.to_str().unwrap();
///     assert_eq!(greeting, "Hello, PortunusD!");
///
///     // We did a naughty and called malloc, so we need to clean up. A PR for a Rustier way
///     // to do this would be considered a personal favor.
///     libc::free(rbuf as *mut libc::c_void);
/// }
/// ```


pub struct DoorHandle {
    path_handle: path::Handle
}


#[derive(Debug)]
pub enum DoorHandleError {
    Handle(path::Error),
    DoorCall(libc::c_int),
    CString(ffi::NulError)
}

impl From<ffi::NulError> for DoorHandleError {
    fn from(error: ffi::NulError) -> Self {
        Self::CString(error)
    }
}

impl From<path::Error> for DoorHandleError {
    fn from(error: path::Error) -> Self {
        Self::Handle(error)
    }
}

/// Talk to Server
///
/// ```
/// use portunusd::define_server_procedure;
/// use portunusd::{door,path,illumos,application_doorway};
/// use std::ffi;
/// use std::fmt::format;
/// use std::str::from_utf8;
/// use std::ptr;
/// use libc;
///
/// define_server_procedure!(Dismiss(name: &[u8]) -> Vec<u8> {
///     match from_utf8(name) {
///         Err(_) => format!("error").into_bytes(),
///         Ok(name) => {
///             let response = format!("Go away, {}!", name);
///             response.into_bytes()
///         }
///     }
/// });
/// let d = door::Door::create(Dismiss::c).unwrap();
/// let j = path::Reservation::make("portunus_516811.door").unwrap();
/// let a = path::Attachment::new(d.descriptor,j).unwrap();
///
/// let dismiss = application_doorway::DoorHandle::open("portunus_516811.door").unwrap();
/// let name = ffi::CString::new("Sunutrop").unwrap();
/// let dismissal = dismiss.call(name.as_bytes()).unwrap();
/// assert_eq!(dismissal, b"Go away, Sunutrop!");
/// ```
///
///
impl DoorHandle {
    /// Open a Door so we can invoke it later.
    ///
    /// If successful, this call returns an object which references a function in another process.
    /// We can use this object to communicate with the remote process, and when it passes out of
    /// scope, its destructor will signal to the operating system to clean up the resources used
    /// for this communication.
    pub fn open(path: &str) -> Result<Self,DoorHandleError> {
        let path_handle = path::Handle::grab(path)?;
        Ok(Self{ path_handle })
    }

    /// Invoke a Server Procedure in another process.
    pub fn call(&self, data: &[u8]) -> Result<Vec<u8>,DoorHandleError> {
        let data_ptr = data.as_ptr() as *const libc::c_char;
        let data_size = data.len();
        let desc_ptr = ptr::null();
        let desc_num = 0;
        let rsize = 4096;
        let mut response = Vec::with_capacity(rsize);
        let rbuf = response.as_ptr() as *const libc::c_char;

        let params = illumos::door_h::door_arg_t {
            data_ptr,
            data_size,
            desc_ptr,
            desc_num,
            rbuf,
            rsize
        };

        match unsafe { illumos::door_h::door_call(self.path_handle.as_c_int(), &params) } {
            -1 => Err(DoorHandleError::DoorCall(illumos::errno())),
            _ => {
                unsafe{ response.set_len(params.data_size); }
                Ok(response)
            }
        }
    }
}
