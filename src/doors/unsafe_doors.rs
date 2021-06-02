// Unsafe Wrapper for the Doors API
//
// This module merely re-exports the subset of the illumos doors api  that we need for this
// project. It makes no attempt at safety or ergonomics. 
//
// Check out https://github.com/robertdfrench/revolving-door for an introduction to doors.


#![allow(non_camel_case_types)]
use libc;


extern "C" {
    // Turns a function into a file descriptor.  See DOOR_CREATE(3C)
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


    // Invokes a function in another process (assuming `d` is a file descriptor for a door which
    // points to a function in another process).  See DOOR_CALL(3C).
    pub fn door_call(d: libc::c_int, params: *const door_arg_t) -> libc::c_int;


    // The inverse of `door_call`. Use this at the end of `server_procedure` in lieu of the
    // traditional `return` statement to transfer control back to the process which originally
    // issued `door_call`. See DOOR_RETURN(3C).
    pub fn door_return(
        data_ptr: *const libc::c_char,
        data_size: libc::size_t,
        desc_ptr: *const door_desc_t,
        num_desc: libc::c_uint,
    ) -> !; // Like EXIT(3C) or EXECVE(2), this function is terminal.


    // Makes a door descriptor visible on the filesystem. Just like sockets must be created (as
    // descriptors) and THEN attached to an IP Address + Port Number by calling BIND(3SOCKET),
    // doors are created (as descriptors) and THEN attached to a path on the filesystem by calling
    // FATTACH(3C).
    pub fn fattach(fildes: libc::c_int, path: *const libc::c_char) -> libc::c_int;


    // Removes a door from the filesystem. See FDETACH(3C).
    pub fn fdetach(path: *const libc::c_char) -> libc::c_int;
}


// This is your daily driver, right here. `data_ptr` and `data_size` represent the bytes you want
// to send to the server. `rbuf` and `rsize` represent a space you've set aside to store bytes that
// come back from the server. `desc_ptr` and `desc_num` are for passing any file / socket / door
// descriptors you'd like the server to be able to access. It is described in more detail below.
#[repr(C)]
pub struct door_arg_t {
    pub data_ptr: *const libc::c_char,
    pub data_size: libc::size_t,
    pub desc_ptr: *const door_desc_t,
    pub desc_num: libc::c_uint,
    pub rbuf: *const libc::c_char,
    pub rsize: libc::size_t,
}


// For our purposes, this data structure and its constituent parts are mostly opaque *except* that
// it holds any file / socket / door descriptors which we would like to pass between processes.
// Rust does not support nested type declaration like C does, so we define each component
// separately. See /usr/include/sys/doors.h for the original (nested) definition of this type and
// https://github.com/robertdfrench/revolving-door/tree/master/A0_result_parameters for a visual
// guide.
#[repr(C)]
pub struct door_desc_t {
    pub d_attributes: door_attr_t,
    pub d_data: door_desc_t__d_data,
}


// Door behavior options, as specified in the "Description" section of DOOR_CREATE(3C).
pub type door_attr_t = libc::c_uint;
pub const DOOR_REFUSE_DESC: door_attr_t = 0x40; // Disable file descriptor passing.


// This is not a real doors data structure *per se*, but rather the `d_data` component of the
// `door_dest_t` type. It is defined in /usr/include/sys/doors.h.
#[repr(C)]
pub union door_desc_t__d_data {
    pub d_desc: door_desc_t__d_data__d_desc,
    d_resv: [libc::c_int; 5], /* Reserved by illumos for some undocumented reason */
}

// This is the `d_desc` component of the `d_data` union of the `door_desct_t` structure. See its
// definition in /usr/include/sys/doors.h.
#[derive(Copy,Clone)]
#[repr(C)]
pub struct door_desc_t__d_data__d_desc {
    pub d_descriptor: libc::c_int,
    pub d_id: door_id_t
}


// Some kind of door identifier. The doors API handles this for us, we don't really need to worry
// about it. Or at least, if I should be worried about it, I'm in a lot of trouble.
pub type door_id_t = libc::c_ulonglong;


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use std::ptr;
    use std::ffi::{CStr,CString};

    #[test]
    fn can_invoke_own_door() {
        // The simplest possible smoke test is to see if we can both call and answer our own door
        // invocation. Remember: door_create does not change control, but door_call and door_return
        // do. So we only need one thread to pull this off.
        extern "C" fn capitalize_string(
            _cookie: *const libc::c_void,
            argp: *const libc::c_char,
            arg_size: libc::size_t,
            _dp: *const door_desc_t,
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
            unsafe { door_return(capitalized.as_ptr(), arg_size, ptr::null(), 0) };
        };

        // Clean up any doors which may still be lingering from a previous test.
        let door_path = Path::new("/tmp/relaydoors_test");
        if door_path.exists() {
            fs::remove_file(door_path).unwrap();
        }
        let door_path_cstring = CString::new(door_path.to_str().unwrap()).unwrap();

        // Create a door for our "Capitalization Server"
        unsafe {
            // Create the (as yet unnamed) door descriptor.
            let server_door_fd = door_create(capitalize_string, ptr::null(), 0);

            // Create an empty file on the filesystem at `door_path`.
            fs::File::create(door_path).unwrap();

            // Give the door descriptor a name on the filesystem.
            fattach(server_door_fd, door_path_cstring.as_ptr());
        }

        // Send an uncapitalized string through the door and see what comes back!
        let original = CString::new("hello world").unwrap();
        unsafe {
            // Connect to the Capitalization Server through its door.
            let client_door_fd = libc::open(door_path_cstring.as_ptr(), libc::O_RDONLY);

            // Pass `original` through the Captialization Server's door.
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

            // This is where the magic happens. We block here while control is transferred to a
            // separate thread which executes `capitalize_string` on our behalf.
            door_call(client_door_fd, &params);

            // Unpack the returned bytes and compare!
            let capitalized = CStr::from_ptr(rbuf);
            let capitalized = capitalized.to_str().unwrap();
            assert_eq!(capitalized, "HELLO WORLD");

            // We did a naughty and called malloc, so we need to clean up. A PR for a Rustier way
            // to do this would be considered a personal favor.
            libc::free(rbuf as *mut libc::c_void);
        }

        // Clean up the door now that we are done.
        unsafe {
            // Make the path no longer be a door (it will go back to being a regular, empty file).
            fdetach(door_path_cstring.as_ptr());

            // Remove the regular, empty file which had been masked by the door.
            fs::remove_file(door_path).unwrap();
        }
    }
}
