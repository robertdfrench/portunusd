use std::io;
use std::net;
use std::os::unix;

use errors::define_error_enum;

use io::Read;
use io::Write;

define_error_enum!(
    pub enum MainError {
        Io(io::Error)
    }
);

fn main() -> Result<(), MainError> {
    let (mut parent, mut child) = unix::net::UnixStream::pair()?;
    match unsafe{ libc::fork() } {
        0 => {
            write!(child, "The darndest things")?;
            parent.shutdown(net::Shutdown::Read)?;
        },
        pid => {
            let mut content = String::from("");
            parent.read_to_string(&mut content)?;
            println!("Process {} says {}", pid, content);
        }
    }

    Ok(())
}
