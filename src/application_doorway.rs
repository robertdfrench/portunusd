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

use crate::jamb;
use crate::door;
use crate::illumos;
use libc;
use std::ffi;
use std::ptr;


pub struct ApplicationDoorway {
    jamb: jamb::Jamb,
    _door: door::Door
}


#[derive(Debug)]
pub enum DoorwayError {
    Jamb(jamb::JambError),
    Door(door::DoorCreationError),
    Fattach(libc::c_int)
}


/// Respond to clients
///
/// ```
/// use portunusd::define_server_procedure;
/// use portunusd::{door,jamb,illumos,application_doorway};
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
/// let j = jamb::Jamb::install("portunusd_b3d839.door").unwrap();
/// let doorway = application_doorway::ApplicationDoorway::create(j,d).unwrap();
///
/// // Pretend to be a client and invoke the Doorway
/// let name = ffi::CString::new("PortunusD").unwrap();
/// unsafe {
///     // Connect to the Capitalization Server through its door.
///     let client_door_fd = libc::open(doorway.path_ptr(), libc::O_RDONLY);
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
impl ApplicationDoorway {
    pub fn create(jamb: jamb::Jamb, door: door::Door) -> Result<Self,DoorwayError> {
        let result = unsafe{ illumos::stropts_h::fattach(door.descriptor, jamb.path.as_ptr()) };
        match result {
            -1 => Err(DoorwayError::Fattach(illumos::errno())),
            _ => Ok(ApplicationDoorway{ jamb, _door: door })
        }
    }

    pub fn path_ptr(&self) -> *const libc::c_char {
        self.jamb.path.as_ptr()
    }
}


impl Drop for ApplicationDoorway {
    fn drop(&mut self) {
        unsafe { illumos::stropts_h::fdetach(self.jamb.path.as_ptr()); }
    }
}


pub struct DoorHandle {
    descriptor: libc::c_int
}


#[derive(Debug)]
pub enum DoorHandleError {
    Open(libc::c_int),
    DoorCall(libc::c_int),
    CString(ffi::NulError)
}

impl From<ffi::NulError> for DoorHandleError {
    fn from(error: ffi::NulError) -> Self {
        Self::CString(error)
    }
}

/// Talk to Server
///
/// ```
/// use portunusd::define_server_procedure;
/// use portunusd::{door,jamb,illumos,application_doorway};
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
/// let j = jamb::Jamb::install("portunus_516811.door").unwrap();
/// let doorway = application_doorway::ApplicationDoorway::create(j,d).unwrap();
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
        let path = ffi::CString::new(path)?;
        let flags = libc::O_RDONLY;
        match unsafe{ libc::open(path.as_ptr(), flags) } {
            -1 => Err(DoorHandleError::Open(illumos::errno())),
            descriptor => Ok(Self{ descriptor })
        }
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

        match unsafe { illumos::door_h::door_call(self.descriptor, &params) } {
            -1 => Err(DoorHandleError::DoorCall(illumos::errno())),
            _ => {
                unsafe{ response.set_len(params.data_size); }
                Ok(response)
            }
        }
    }
}

impl Drop for DoorHandle {
    /// Automatically prevent clients from calling the door when it goes out of scope.
    fn drop(&mut self) {
        unsafe{ libc::close(self.descriptor); }
    }
}
