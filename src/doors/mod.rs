mod casing;
mod jamb;

use std::fs::File;
use std::os::unix::io::FromRawFd;
use std::path::Path;


pub struct Door {
    location: jamb::Jamb,
    knob: File
}

impl Door {
    pub fn hang<P: AsRef<Path>>(path: P, f: casing::door_server_procedure_t) {
    }
}
