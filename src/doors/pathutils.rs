use libc;
use std::ffi;

// It's where you hang a door: https://www.hgtv.com/how-to/home-improvement/how-to-hang-a-door
pub struct Jamb {
    path: ffi::CString,
    descriptor: libc::c_int
}

#[derive(Debug)]
pub enum JambError {
    Open(libc::c_int)
}

impl Jamb {
    pub fn install(path: &ffi::CStr) -> Result<Self,JambError> {
        let path = path.to_owned();
        // Per https://www.reddit.com/r/illumos/comments/babxsl/doors_api_tutorial/eke7es9/
        let flags = libc::O_RDWR | libc::O_CREAT | libc::O_EXCL;
        match unsafe{ libc::open(path.as_ptr(), flags, 0400) } {
            -1 => Err(JambError::Open(unsafe{ *libc::___errno() })),
            descriptor => Ok(Self{ path, descriptor })
        }
    }
}

impl Drop for Jamb {
    fn drop(&mut self) {
        // We're dropping, so just ignore errors and hope for the best. Maybe a better strategy
        // would be to post a message to syslog if either of these fail? Or just panic?
        unsafe{ libc::close(self.descriptor) };
        unsafe{ libc::unlink(self.path.as_ptr()) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn exists(path: &ffi::CStr) -> bool {
        let result = unsafe { libc::access(path.as_ptr(), libc::F_OK) };
        result == 0
        // ^ this isn't entirely accurate. access(2) could have failed for some reason (possibly
        // permissions) which would make this call return false even though the file exists.
    }

    #[test]
    fn can_install_jamb() {
        // Create the door at this location
        let path = ffi::CString::new("./test.door").unwrap();

        // Start out by verifying that this path is available for our door jamb
        assert_eq!(exists(&path), false);

        {
            // Create the jamb inside a limited scope, and demonstrate that it is detectable on the
            // filesystem.
            let jamb = Jamb::install(&path).unwrap();
            assert_eq!(exists(&path), true);
        }

        // This implies that our destructor ran successfully
        assert_eq!(exists(&path), false);
    }

    #[test]
    #[should_panic]
    fn cannot_install_jamb_when_something_is_in_the_way() {
        let path = ffi::CString::new(".").unwrap();
        Jamb::install(&path).unwrap(); // This path already exists
    }
}
