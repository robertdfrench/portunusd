/* 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2021 Robert D. French
 */
//! Abstract descriptor for a file or a door.


use libc;


/// Not a file, just a descriptor. Nearly 100% guaranteed to close.
///
/// Dealing with doors requires us to do a few *file-like* things that could be sortof awkward with
/// Rust's [`File`] API. We boil this down to the minimum necessary abstraction -- the ability to
/// [`OPEN(2)`] (and automatically [`CLOSE(2)`]) a filesystem path.
///
/// Note: you can't actually guarantee that a descriptor will be closed when you ask, that's up to
/// the kernel. See the "RETURN VALUES" section of [`CLOSE(2)`] for more details.
///
/// [`OPEN(2)`]: https://illumos.org/man/2/open
/// [`CLOSE(2)`]: https://illumos.org/man/2/close
pub struct Descriptor {
    pub raw_fd: libc::c_int
}


impl Descriptor {
    /// To be compatible with libc
    pub fn as_c_int(&self) -> libc::c_int {
        self.raw_fd
    }
}


/// Try to close the descriptor when we go out of scope
///
/// If close is interrupted by a signal, try again. If close failed for some other reason, just
/// panic. This strategy should be regarded as suspicious.
impl Drop for Descriptor {
    fn drop(&mut self) {
        unsafe{ libc::close(self.raw_fd); }
    }
}


/// Use this to keep track of descriptors created through other means (like doors)
impl From<libc::c_int> for Descriptor {
    fn from(raw_fd: libc::c_int) -> Self {
        Self{ raw_fd }
    }
}
