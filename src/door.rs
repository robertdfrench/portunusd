/* 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2021 Robert D. French
 */
//! A live Server Procedure


use crate::illumos::door_h::{
    door_create,
    door_server_procedure_t,
    DOOR_REFUSE_DESC
};
use crate::illumos::errno;
use crate::descriptor;
use libc;
use std::ptr;


/// An actual, running Door
///
/// This type represents a running Door function based on your derived server procedure type. It
/// isn't visible on the filesystem yet (we'll do that in [`ApplicationDoorway`]) but theoretically
/// it could respond to [`DOOR_CALL(3C)`]s issued by an application which had otherwise been given
/// access to this door (say, by passing it over a socket or a different door).
pub struct Door {
    pub descriptor: descriptor::Descriptor,
}


/// The underlying `errno` when a door can't be created.
#[derive(Debug)]
pub struct DoorCreationError(libc::c_int);


impl Door {
    /// Create a callable door descriptor from a server procedure
    ///
    /// Given a server procedure `function`, call [`DOOR_CREATE(3C)`] to get a descriptor which can
    /// be used to give other processes the ability to invoke `function`. For Portunus
    /// Applications, this descriptor will later be advertised on the filesystem by calling
    /// [`FATTACH(3C)`].
    ///
    /// [`FATTACH(3C)`]: https://illumos.org/man/3c/fattach
    /// [`DOOR_CREATE(3C)`]: https://illumos.org/man/3c/door_create
    pub fn create(function: door_server_procedure_t) -> Result<Self,DoorCreationError> {
        let result = unsafe { door_create(function, ptr::null(), DOOR_REFUSE_DESC) };
        match result {
            -1 => Err(DoorCreationError(errno())),
            descriptor => Ok(Door{ descriptor: descriptor.into() })
        }
    }

    /// Use the door in a doors API call
    pub fn as_c_int(&self) -> libc::c_int {
        self.descriptor.as_c_int()
    }
}
