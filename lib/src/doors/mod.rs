mod native;
mod pathutils;

use libc;
use pathutils::{Jamb,JambError};
use std::ptr;
use std::ffi;
use std::convert::From;


pub struct Door {
    jamb: Jamb,
    descriptor: libc::c_int
}

#[derive(Debug)]
pub enum DoorError {
    Jamb(JambError),
    DoorCreate(libc::c_int),
    Fattach(libc::c_int)
}

impl From<JambError> for DoorError {
    fn from(error: JambError) -> Self {
        DoorError::Jamb(error)
    }
}

impl Door {
    pub fn hang(path: &ffi::CStr, f: native::door_server_procedure_t) -> Result<Self,DoorError> {
        let jamb = Jamb::install(&path)?;
        let descriptor = Self::create(f)?;
        match unsafe{ native::fattach(descriptor, path.as_ptr()) } {
            -1 => Err(DoorError::Fattach(unsafe{ *libc::___errno() })),
            _ => Ok(Self{ jamb, descriptor })
        }
    }

    fn create(f: native::door_server_procedure_t) -> Result<libc::c_int,DoorError> {
        match unsafe { native::door_create(f, ptr::null(), 0) } {
            -1 => Err(DoorError::DoorCreate(unsafe{ *libc::___errno() })),
            descriptor => Ok(descriptor)
        }
    }
}

impl Drop for Door {
    fn drop(&mut self) {
        unsafe{ libc::close(self.descriptor) };
    }
}
