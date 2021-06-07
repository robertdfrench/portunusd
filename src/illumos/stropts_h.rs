/* 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2021 Robert D. French
 */

//! Unsafe Declarations for the illumos STREAMS API
//!
//! This module merely re-exports the subset of the illumos STREAMS api that we need for this
//! project. It makes no attempt at safety or ergonomics.
//!
//! While STREAMS are not strictly relevant to this project, some of their features are overloaded
//! to work with doors. Those are the bits we redefine here.

extern "C" {
    /// Makes a door descriptor visible on the filesystem.
    ///
    /// Just like sockets must be created (as descriptors) and *then* attached to an IP Address +
    /// Port Number by calling [`BIND(3SOCKET)`], doors are created (as descriptors) and *then*
    /// attached to a path on the filesystem by calling [`FATTACH(3C)`].
    ///
    /// [`BIND(3SOCKET)`]: https://illumos.org/man/3socket/bind
    /// [`FATTACH(3C)`]: https://illumos.org/man/3c/fattach
    pub fn fattach(fildes: libc::c_int, path: *const libc::c_char) -> libc::c_int;
}
