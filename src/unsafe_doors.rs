#![allow(non_camel_case_types)]
use libc;


pub type door_attr_t = libc::c_uint;
pub type door_id_t = libc::c_ulonglong;

#[derive(Copy,Clone)]
#[repr(C)]
pub struct door_desc_t__d_data__d_desc {
    pub d_descriptor: libc::c_int,
    pub d_id: door_id_t
}

#[repr(C)]
pub struct door_desc_t {
    pub d_attributes: door_attr_t,
    pub d_data: door_desc_t__d_data,
}

#[repr(C)]
pub union door_desc_t__d_data {
    pub d_desc: door_desc_t__d_data__d_desc,
    d_resv: [libc::c_int; 5], /* Check out /usr/include/sys/door.h */
}

#[repr(C)]
pub struct door_arg_t {
    pub data_ptr: *const libc::c_char,
    pub data_size: libc::size_t,
    pub desc_ptr: *const door_desc_t,
    pub desc_num: libc::c_uint,
    pub rbuf: *const libc::c_char,
    pub rsize: libc::size_t,
}

extern "C" {
    pub fn door_create(
        server_procedure: extern "C" fn(
            cookie: *const libc::c_void,
            argp: *const libc::c_char,
            arg_size: libc::size_t,
            dp: *const door_desc_t,
            n_desc: libc::c_uint,
        ),
        cookie: *const libc::c_void,
        attributes: door_attr_t,
    ) -> libc::c_int;

    pub fn door_call(
        d: libc::c_int,
        params: *const door_arg_t
    ) -> libc::c_int;

    pub fn door_return(
        data_ptr: *const libc::c_char,
        data_size: libc::size_t,
        desc_ptr: *const door_desc_t,
        num_desc: libc::c_uint,
    ) -> !;

    pub fn fattach(
        fildes: libc::c_int,
        path: *const libc::c_char,
    ) -> libc::c_int;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;
    use std::ffi::{CStr,CString};

    #[test]
    fn can_invoke_own_door() {
        extern "C" fn capitalize_string(
            _cookie: *const libc::c_void,
            argp: *const libc::c_char,
            arg_size: libc::size_t,
            _dp: *const door_desc_t,
            _n_desc: libc::c_uint,
        ) {
            // Capitalize the string provided by the client. This
            // is a lazy way to verify that we are able to send and
            // receive data through doors.
            let original = unsafe { CStr::from_ptr(argp) };
            let original = original.to_str().unwrap();
            let capitalized = original.to_ascii_uppercase();
            let capitalized = CString::new(capitalized).unwrap();
            unsafe {
                door_return(
                    capitalized.as_ptr(),
                    arg_size,
                    ptr::null(),
                    0
                );
            }
        };
        let original = CString::new("hello world").unwrap();
        let path = CString::new("/var/run/relaydoors_test_door").
            unwrap();
        unsafe {
            // Set up server
            let server_door_fd = door_create(
                capitalize_string, ptr::null(), 0
            );
            let path_fd = libc::open(
                path.as_ptr(),
                libc::O_RDWR | libc::O_CREAT | libc::O_EXCL,
                0400
            );
            libc::close(path_fd);
            fattach(server_door_fd, path.as_ptr());

            // Invoke server procedure
            let data_ptr = original.as_ptr();
            let data_size = 12;

            let desc_ptr = ptr::null();
            let desc_num = 0;

            let rbuf = libc::malloc(data_size) as *mut libc::c_char;
            let rsize = data_size;

            let params = door_arg_t {
                data_ptr,
                data_size,
                desc_ptr,
                desc_num,
                rbuf,
                rsize
            };
            let client_door_fd = libc::open(
                path.as_ptr(),
                libc::O_RDONLY
            );

            door_call(client_door_fd, &params);
            let capitalized = CStr::from_ptr(rbuf);
            let capitalized = capitalized.to_str().unwrap();
            assert_eq!(capitalized, "HELLO WORLD");

            libc::free(rbuf as *mut libc::c_void);
        }
    }
}
