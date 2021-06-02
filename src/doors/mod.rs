use libc;
use std::os::unix::io::RawFd;
use std::path::Path;
use std::ptr;
mod unsafe_doors;

pub type ServerProcedure = extern "C" fn(
	cookie: *const libc::c_void,
	argp: *const libc::c_char,
	arg_size: libc::size_t,
	dp: *const unsafe_doors::door_desc_t,
	n_desc: libc::c_uint
);


pub struct DoorFrame {
    descriptor: RawFd
}

impl DoorFrame {
    pub fn new(path: &Path, server_procedure: ServerProcedure) -> Self {
        let descriptor = unsafe {
            unsafe_doors::door_create(server_procedure, ptr::null(), 0)
        };
        Self { descriptor }
    }
}
