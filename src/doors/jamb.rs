use std::io::Result;
use std::fs::{self,OpenOptions};
use std::path::{Path,PathBuf};

// It's where you hang a door: https://www.hgtv.com/how-to/home-improvement/how-to-hang-a-door
pub struct Jamb {
    path: PathBuf
}

impl Jamb {
    pub fn install<P: AsRef<Path>>(path: P) -> Result<Self> {
        // Per https://www.reddit.com/r/illumos/comments/babxsl/doors_api_tutorial/eke7es9/ door
        // paths should be reserved with O_RDWR | O_CREAT | O_EXCL which translates to this:
        let _file = OpenOptions::new().read(true).write(true).create_new(true).open(&path)?;

        // We purposefully ignore the _file ^ variable because we want Rust to Drop (and therefore
        // close(2)) it before we make use of it as a door jamb.
        let path = path.as_ref().to_owned();
        Ok(Self { path })
    }
}

impl Drop for Jamb {
    fn drop(&mut self) {
        fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_install_jamb() {
        let path = Path::new("empty_file_test");
        assert_eq!(path.exists(), false);

        {
            let _jamb = Jamb::install(path).unwrap();
            assert_eq!(path.exists(), true);
        }

        assert_eq!(path.exists(), false);
    }

    #[test]
    #[should_panic]
    fn cannot_install_jamb_when_something_is_in_the_way() {
        let path = Path::new("."); // This path already exists
        Jamb::install(path).unwrap();
    }
}
