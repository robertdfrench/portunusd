use std::fs;
use std::io;
use std::net;
use std::os::unix;

use errors::define_error_enum;

use io::Read;
use std::os::fd::FromRawFd;
use std::os::fd::IntoRawFd;
use sendfd::SendWithFd;
use sendfd::RecvWithFd;

use std::{thread, time};


define_error_enum!(
    pub enum MainError {
        Io(io::Error)
    }
);

fn main() -> Result<(), MainError> {
    let (parent, child) = unix::net::UnixStream::pair()?;
    parent.shutdown(net::Shutdown::Write)?;
    child.shutdown(net::Shutdown::Read)?;
    match unsafe{ libc::fork() } {
        0 => {
            let file = fs::File::open("Cargo.toml")?;
            child.send_with_fd(&vec![1], &vec![file.into_raw_fd()])?;
            parent.shutdown(net::Shutdown::Read)?;
            child.shutdown(net::Shutdown::Write)?;
            println!("Child is Done");
        },
        pid => {
            let mut data = vec![0];
            let mut fds = vec![];
            thread::sleep(time::Duration::from_millis(1000));
            parent.recv_with_fd(&mut data, &mut fds)?;
            let mut content = String::from("");
            if fds.len() > 0 {
                let mut file = unsafe{ fs::File::from_raw_fd(fds[0]) };
                file.read_to_string(&mut content)?;
            }
            println!("Process {} provided ({:?},{:?}) and this file: {}", pid, data,fds, content);
        }
    }

    Ok(())
}
