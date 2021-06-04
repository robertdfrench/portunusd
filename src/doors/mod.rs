mod casing;
mod jamb;

use std::fs::File;
use std::io::{Error, ErrorKind, Result};
use std::os::unix::io::FromRawFd;
use std::path::Path;
use std::ptr;


pub struct Door {
    location: jamb::Jamb,
    knob: File
}

impl Door {
    pub fn hang<P: AsRef<Path>>(path: P, f: casing::door_server_procedure_t) -> Result<Self> {
        let location = jamb::Jamb::install(path)?;
        let descriptor = unsafe { casing::door_create(f, ptr::null(), 0) };
        let filename = location.to_str().ok_or(Error::new(ErrorKind::Other, "garbled path"))?;
        unsafe { casing::fattach(descriptor, filename.as_ptr() as *const i8); }
        let knob = unsafe { File::from_raw_fd(descriptor) };
        Ok(Door{ location, knob })
    }
}

impl Drop for Door {
    fn drop(&mut self) {
        // At this point (dropping) we know the filename will unwrap safely, because it did so
        // while creating the file.
        let filename = self.location.to_str().unwrap();
        unsafe { casing::fdetach(filename.as_ptr() as *const i8); }
    }
}
