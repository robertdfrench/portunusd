/* 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2021 Robert D. French
 */
//! It's where you hang a door.
//!
//! According to [hgtv], the "Jamb" is the part of the door system where the hinges are mounted,
//! and is thus the support system from which we can suspend (or "hang") a door. Stretching this
//! analogy obnoxiously far, a jamb is a structure which, when successfully constructed,
//! guarantees a filesystem entry appropriate for [fattach]ing to a door descriptor.
//!
//! [hgtv]: https://www.hgtv.com/how-to/home-improvement/how-to-hang-a-door
//! [fattach]: ../illumos/stropts_h/fn.fattach.html


use crate::illumos;
use libc;
use std::ffi;


/// basically just an open path...
///
/// You can think of the jamb as *reserving* a spot on the filesystem for the door, or acting as a
/// support where we will install the door later. From its own perspective, the jamb is just a
/// regular, empty file (it doesn't actually know what a door is).
pub struct Jamb {
    pub path: ffi::CString,
    descriptor: libc::c_int
}


/// Trouble hanging the jamb?
///
/// You gave it a bad path. The value of the `JambError` will tell you why the path is bad. Could be
/// that you gave it a path full of naughty characters, could be that your disk is out to lunch and
/// all paths are temporarily bad. Who knows what things can happen on a computer.
#[derive(Debug)]
pub enum JambError {
    Open(libc::c_int),
    CString(ffi::NulError)
}

impl From<ffi::NulError> for JambError {
    fn from(error: ffi::NulError) -> Self {
        JambError::CString(error)
    }
}


impl Jamb {
    /// Reserve a place on the filesystem from which we can later hang a Door.
    ///
    /// In general, `path` should point to a spot on the filesystem that doesn't currently exist,
    /// and to which you have permission to write. Once the jamb is created, an empty file will
    /// exist at this path (we will later turn this into a Door, but that's for another module).
    pub fn install(path: &str) -> Result<Self,JambError> {
        let path = ffi::CString::new(path)?;
        // Per https://www.reddit.com/r/illumos/comments/babxsl/doors_api_tutorial/eke7es9/
        let flags = libc::O_RDWR | libc::O_CREAT | libc::O_EXCL;
        match unsafe{ libc::open(path.as_ptr(), flags, 0400) } {
            -1 => Err(JambError::Open(illumos::errno())),
            descriptor => Ok(Self{ path, descriptor })
        }
    }
}


impl Drop for Jamb {
    /// Automatically remove the jamb from the filesystem when it goes out of scope.
    fn drop(&mut self) {
        // We're dropping, so just ignore errors and hope for the best. Maybe a better strategy
        // would be to post a message to syslog if either of these fail? Or just panic?
        unsafe{ libc::close(self.descriptor) };
        unsafe{ libc::unlink(self.path.as_ptr()) };
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
            // ^ this isn't entirely accurate. access(2) could have failed for some reason (possibly
            // permissions) which would make this call return false even though the file exists.
        }
    }

    #[test]
    fn can_install_jamb() {
        // Create the door at this location
        let path = "./test.door";

        // Start out by verifying that this path is available for our door jamb
        assert_eq!(exists(&path), false);

        {
            // Create the jamb inside a limited scope, and demonstrate that it is detectable on the
            // filesystem.
            let _jamb = Jamb::install(&path).unwrap();
            assert_eq!(exists(&path), true);
        }

        // This implies that our destructor ran successfully
        assert_eq!(exists(&path), false);
    }

    #[test]
    #[should_panic]
    fn cannot_install_jamb_when_something_is_in_the_way() {
        let path = ".";
        Jamb::install(&path).unwrap(); // This path already exists
    }
}
