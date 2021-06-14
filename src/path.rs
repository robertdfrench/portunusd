/* 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2021 Robert D. French
 */
//! Open (and close!) unusual objects on the filesystem


use crate::illumos;
use crate::descriptor::Descriptor;
use libc;
use std::ffi;


/// A self-closing reference to a filesystem object
pub struct Handle {
    descriptor: Descriptor
}


impl Handle {
    /// Attempt to grab a handle to a path which already exists
    pub fn grab(path: &str) -> Result<Self, Error> {
        let open_path = OpenPath::new(path, libc::O_RDONLY, 0)?;
        Ok(Self{ descriptor: open_path.descriptor })
    }

    /// Use this when calling `libc` functions that need a descriptor
    pub fn as_c_int(&self) -> libc::c_int {
        self.descriptor.raw_fd
    }
}


/// A self-deleting (gauranteed new!) empty file
pub struct Reservation {
    pub path: ffi::CString
}


impl Reservation {
    /// Claim a spot on the filesystem, or die trying
    pub fn make(path: &str) -> Result<Self, Error> {
        let flags = libc::O_RDWR | libc::O_CREAT | libc::O_EXCL; // Fail if path already exists
        let open_path = OpenPath::new(path, flags, 0400)?;
        Ok(Self{ path: open_path.raw_string })
    }
}


impl Drop for Reservation {
    fn drop(&mut self) {
        unsafe{ libc::unlink(self.path.as_ptr()) };
    }
}


/// A non-file descriptor attached to a file-looking path.
pub struct Attachment {
    reservation: Reservation,
    _descriptor: Descriptor
}

impl Attachment {
    pub fn new(descriptor: Descriptor, reservation: Reservation) -> Result<Self,Error> {
        let raw_fd = descriptor.as_c_int();
        let path_ptr = reservation.path.as_ptr();
        match unsafe{ illumos::stropts_h::fattach(raw_fd, path_ptr) } {
            -1 => Err(Error::FailedToAttach(illumos::errno())),
            _ => Ok(Self{ reservation, _descriptor: descriptor })
        }
    }

    pub fn path_ptr(&self) -> *const libc::c_char {
        self.reservation.path.as_ptr()
    }
}
    
impl Drop for Attachment {
    fn drop(&mut self) {
        unsafe{ illumos::stropts_h::fdetach(self.path_ptr()); }
    }
}

/// A path which is guaranteed to be open.
///
/// This is only for use by types defined in this module.
struct OpenPath {
    pub raw_string: ffi::CString,
    pub descriptor: Descriptor
}


impl OpenPath {
    pub fn new(path: &str, oflag: libc::c_int, mode: libc::mode_t) -> Result<Self, Error> {
        let raw_string = ffi::CString::new(path)?;
        let descriptor = Self::safe_open(&raw_string, oflag, mode)?;
        Ok(Self{ raw_string, descriptor })
    }

    /// A safe wrapper for [`OPEN(2)`].
    ///
    /// [`OPEN(2)`]: https://illumos.org/man/2/open
    fn safe_open(path: &ffi::CStr, oflag: libc::c_int, mode: libc::mode_t)
        -> Result<Descriptor, Error> {
        match unsafe{ libc::open(path.as_ptr(), oflag, mode) } {
            -1 => Err(Error::FailedToOpen(illumos::errno())),
            raw_fd => Ok(raw_fd.into())
        }
    }
}


/// Things that can go wrong with path creation
#[derive(Debug)]
pub enum Error {
    FailedToOpen(libc::c_int),
    InvalidCString(ffi::NulError),
    FailedToAttach(libc::c_int)
}


impl From<ffi::NulError> for Error {
    fn from(error: ffi::NulError) -> Self {
        Self::InvalidCString(error)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    
    fn exists(path: &str) -> bool {
        match ffi::CString::new(path) {
            Err(_) => false,
            Ok(path) => {
                let result = unsafe { libc::access(path.as_ptr(), libc::F_OK) };
                result == 0
            }
        }
    }

    #[test]
    fn can_install_jamb() {
        // Create the door at this location
        let path = "portunusd_d547f8.test";

        // Start out by verifying that this path is available for our door jamb
        assert_eq!(exists(&path), false);

        {
            // Create the jamb inside a limited scope, and demonstrate that it is detectable on the
            // filesystem.
            let _jamb = Reservation::make(&path).unwrap();
            assert_eq!(exists(&path), true);
        }

        // This implies that our destructor ran successfully
        assert_eq!(exists(&path), false);
    }

    #[test]
    #[should_panic]
    fn cannot_install_jamb_when_something_is_in_the_way() {
        Reservation::make(".").unwrap(); // This path already exists
    }
}
