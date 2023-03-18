/* 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2021 Robert D. French
 */

//! illumos-specific APIs not (yet) found in the libc crate
//!
//! Portunus makes heavy use of illumos' [doors] facility, a novel IPC system that resembles UNIX
//! domain sockets but allows for *much* faster switching between client and server contexts.
//! Because of its obscurity and sharp corners, there is not yet a full representation of the doors
//! API in the [libc] crate.
//!
//! In this module, we represent only the subset of the illumos-specific APIs that we need for
//! Portunus.
//!
//! [doors]: https://github.com/robertdfrench/revolving-door#revolving-doors
//! [libc]: https://github.com/rust-lang/libc/tree/master/src/unix/solarish

pub mod door_h;
pub mod stropts_h;

use libc;
use std::os::fd;
use std::os::fd::AsRawFd;

/// Good ole UNIX errno
///
/// `errno` is implemented in the libc crate, sortof. In real life, it's allegedly a macro or
/// something. Can't be bothered to look it up. Point is, once we've done a goof, we call this to
/// figure out which goof we've done.
///
/// See [`PERROR(3C)`], but don't think too hard about the fact that this is a function and that
/// one doesn't seem to be.
///
/// [`PERROR(3C)`]: https://illumos.org/man/3c/errno
pub fn errno() -> libc::c_int {
    unsafe{ *libc::___errno() }
}

impl AsRawFd for door_h::door_desc_t {
    fn as_raw_fd(&self) -> fd::RawFd {
        let d_data = &self.d_data;
        let d_desc = unsafe{ d_data.d_desc };
        let d_descriptor = d_desc.d_descriptor;
        d_descriptor as fd::RawFd
    }
}

impl door_h::door_desc_t {
    pub fn new(raw: fd::RawFd, release: bool) -> Self {
        let d_descriptor = raw as libc::c_int;
        let d_id = 0; // TODO: Confirm "door 0" is appropriate / not wrong for passing sockets
        let d_desc = door_h::door_desc_t__d_data__d_desc { d_descriptor, d_id };
        let d_data = door_h::door_desc_t__d_data { d_desc };

        let d_attributes = match release {
            false => door_h::DOOR_DESCRIPTOR,
            true  => door_h::DOOR_DESCRIPTOR | door_h::DOOR_RELEASE
        };
        Self { d_attributes, d_data }
    }

    pub fn will_release(&self) -> bool {
        self.d_attributes == (door_h::DOOR_DESCRIPTOR | door_h::DOOR_RELEASE)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use std::ptr;
    use std::ffi::{CStr,CString};

    #[test]
    fn errno_works() {
        // This test will purposefully open a nonexistant file via the libc crate, and then check
        // that errno is the expected value.
        let badpath = CString::new("<(^_^)>").unwrap();
        assert_eq!(unsafe{ libc::open(badpath.as_ptr(), libc::O_RDONLY) }, -1);
        assert_eq!(errno(), libc::ENOENT);
    }

    #[test]
    fn can_invoke_own_door() {
        // The simplest possible smoke test is to see if we can both call and answer our own door
        // invocation. Remember: door_create does not change control, but door_call and door_return
        // do. So we only need one thread to pull this off.
        extern "C" fn capitalize_string(
            _cookie: *const libc::c_void,
            argp: *const libc::c_char,
            arg_size: libc::size_t,
            _dp: *const door_h::door_desc_t,
            _n_desc: libc::c_uint,
        ) {
            // Capitalize the string provided by the client. This is a lazy way to verify that we
            // are able to send and receive data through doors. We aren't testing descriptors,
            // because we aren't really testing doors itself, just making sure our Rust interface
            // works.
            let original = unsafe { CStr::from_ptr(argp) };
            let original = original.to_str().unwrap();
            let capitalized = original.to_ascii_uppercase();
            let capitalized = CString::new(capitalized).unwrap();
            unsafe { door_h::door_return(capitalized.as_ptr(), arg_size, ptr::null(), 0) };
        }

        // Clean up any doors which may still be lingering from a previous test.
        let door_path = Path::new("/tmp/relaydoors_test_f431a5");
        if door_path.exists() {
            fs::remove_file(door_path).unwrap();
        }
        let door_path_cstring = CString::new(door_path.to_str().unwrap()).unwrap();

        // Create a door for our "Capitalization Server"
        unsafe {
            // Create the (as yet unnamed) door descriptor.
            let server_door_fd = door_h::door_create(capitalize_string, ptr::null(), 0);

            // Create an empty file on the filesystem at `door_path`.
            fs::File::create(door_path).unwrap();

            // Give the door descriptor a name on the filesystem.
            stropts_h::fattach(server_door_fd, door_path_cstring.as_ptr());
        }

        // Send an uncapitalized string through the door and see what comes back!
        let original = CString::new("hello world").unwrap();
        unsafe {
            // Connect to the Capitalization Server through its door.
            let client_door_fd = libc::open(door_path_cstring.as_ptr(), libc::O_RDONLY);

            // Pass `original` through the Capitalization Server's door.
            let data_ptr = original.as_ptr();
            let data_size = 12;
            let desc_ptr = ptr::null();
            let desc_num = 0;
            let rbuf = libc::malloc(data_size) as *mut libc::c_char;
            let rsize = data_size;

            let params = door_h::door_arg_t {
                data_ptr,
                data_size,
                desc_ptr,
                desc_num,
                rbuf,
                rsize
            };

            // This is where the magic happens. We block here while control is transferred to a
            // separate thread which executes `capitalize_string` on our behalf.
            door_h::door_call(client_door_fd, &params);

            // Unpack the returned bytes and compare!
            let capitalized = CStr::from_ptr(rbuf);
            let capitalized = capitalized.to_str().unwrap();
            assert_eq!(capitalized, "HELLO WORLD");

            // We did a naughty and called malloc, so we need to clean up. A PR for a Rustier way
            // to do this would be considered a personal favor.
            libc::free(rbuf as *mut libc::c_void);
        }

        // Clean up the door now that we are done.
        fs::remove_file(door_path).unwrap();
    }

    #[test]
    fn retain_door_desc_t() {
        let dd = door_h::door_desc_t::new(-1, false);
        assert!(!dd.will_release());
    }

    #[test]
    fn release_door_desc_t() {
        let dd = door_h::door_desc_t::new(-1, true);
        assert!(dd.will_release());
    }
}
