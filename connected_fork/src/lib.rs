/* 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2023 Robert D. French
 */

// Types
use std::os::fd::RawFd;

// Traits
use std::os::fd::FromRawFd;
use std::os::fd::AsRawFd;

// Macros
use errors::define_error_enum;

pub struct RecvFd(illumos::stropts_h::strrecvfd);

impl AsRawFd for RecvFd {
    fn as_raw_fd(&self) -> RawFd {
        self.0.fd
    }
}

pub struct PipeEnd {
    fd: RawFd
}

impl FromRawFd for PipeEnd {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self{ fd }
    }
}

#[derive(Debug,PartialEq)]
pub enum PipeCloseError {
    EBADF,
    EINTR,
    ENOLINK,
    ENOSPC,
    EIO
}

#[derive(Debug)]
pub enum SendFdError {
    EAGAIN,
    EBADF,
    EINVAL,
    ENXIO
}

#[derive(Debug)]
pub enum RecvFdError {
    EAGAIN,
    EBADMSG,
    EFAULT,
    EMFILE,
    ENXIO,
    EOVERFLOW
}

impl PipeEnd {
    pub fn close(&mut self) -> Result<(), PipeCloseError> {
        match unsafe{ libc::close(self.fd) } {
            0 => Ok(()),
            _ => match illumos::errno() {
                libc::EBADF => Err(PipeCloseError::EBADF),
                libc::EINTR => Err(PipeCloseError::EINTR),
                libc::ENOLINK => Err(PipeCloseError::ENOLINK),
                libc::ENOSPC => Err(PipeCloseError::ENOSPC),
                libc::EIO => Err(PipeCloseError::EIO),
                _ => unreachable!()
            }
        }
    }

    pub fn send_fd(&mut self, fd: RawFd) -> Result<(), SendFdError> {
        match unsafe{ libc::ioctl(self.fd, libc::I_SENDFD, fd) } {
            0 => Ok(()),
            _ => match illumos::errno() {
                libc::EAGAIN => Err(SendFdError::EAGAIN),
                libc::EBADF => Err(SendFdError::EBADF),
                libc::EINVAL => Err(SendFdError::EINVAL),
                libc::ENXIO => Err(SendFdError::ENXIO),
                _ => unreachable!()
            }
        }
    }

    pub fn recv_fd(&mut self) -> Result<RecvFd, RecvFdError> {
        let mut received: Vec<illumos::stropts_h::strrecvfd> = Vec::with_capacity(1);
        match unsafe{ libc::ioctl(self.fd, libc::I_RECVFD, received.as_mut_ptr()) } {
            0 => {
                unsafe{ received.set_len(1) };
                Ok(RecvFd(received.pop().unwrap()))
            },
            _ => match illumos::errno() {
                libc::EAGAIN => Err(RecvFdError::EAGAIN),
                libc::EBADMSG => Err(RecvFdError::EBADMSG),
                libc::EFAULT => Err(RecvFdError::EFAULT),
                libc::EMFILE    => Err(RecvFdError::EMFILE),
                libc::ENXIO     => Err(RecvFdError::ENXIO),
                libc::EOVERFLOW => Err(RecvFdError::EOVERFLOW),
                _ => unreachable!()
            }
        }
    }
}

impl Drop for PipeEnd {
    fn drop(&mut self) {
        while let Err(e) = self.close() {
            if e == PipeCloseError::EINTR {
                continue;
            }
        }
    }
}

#[derive(Debug)]
pub enum PipeOpenError {
    EMFILE,
    ENFILE,
    EFAULT
}


pub fn pipe() -> Result<(PipeEnd,PipeEnd), PipeOpenError> {
    let mut fds: Vec<RawFd> = vec![0; 2];
    match unsafe{ libc::pipe(fds.as_mut_ptr()) } {
        0 => {
            let child = unsafe{ PipeEnd::from_raw_fd(fds[0]) };
            let parent = unsafe{ PipeEnd::from_raw_fd(fds[1]) };
            Ok((parent, child))
        },
        _ => {
            match illumos::errno() {
                libc::EMFILE => Err(PipeOpenError::EMFILE),
                libc::ENFILE => Err(PipeOpenError::ENFILE),
                libc::EFAULT => Err(PipeOpenError::EFAULT),
                _ => unreachable!()
            }
        }
    }
}

pub enum Fork {
    Parent(libc::pid_t),
    Child
}

#[derive(Debug)]
pub enum ForkError {
    EAGAIN,
    ENOMEM,
    EPERM
}

impl Fork {
    pub fn new() -> Result<Self, ForkError> {
        match unsafe{ libc::fork() } {
            0 => Ok(Fork::Child),
            pid => {
                if pid > 0 {
                    Ok(Fork::Parent(pid))
                } else {
                    match pid {
                        libc::EAGAIN => Err(ForkError::EAGAIN),
                        libc::ENOMEM => Err(ForkError::ENOMEM),
                        libc::EPERM => Err(ForkError::EPERM),
                        _ => unreachable!()
                    }
                }
            }
        }
    }
}

define_error_enum!(
    pub enum ConnectedForkError {
        PipeClose(PipeCloseError),
        SendFd(SendFdError),
        RecvFd(RecvFdError),
        PipeOpen(PipeOpenError),
        Fork(ForkError)
    }
);

pub enum ConnectedFork {
    Parent(libc::pid_t, PipeEnd),
    Child(PipeEnd)
}

impl ConnectedFork {
    pub fn with_creds(uid: libc::uid_t, gid: libc::gid_t) -> Result<Self, ConnectedForkError> {
        let (parent, child) = pipe()?;
        match Fork::new()? {
            Fork::Parent(pid) => {
                drop(parent);
                Ok(Self::Parent(pid, child))
            },
            Fork::Child => {
                drop(child);
                if unsafe{ libc::setgid(gid) } != 0 { std::process::exit(illumos::errno()); }
                if unsafe{ libc::setuid(uid) } != 0 { std::process::exit(illumos::errno()); }
                Ok(Self::Child(parent))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::io::Read;

    #[test]
    fn it_works() {
        let (_parent, _child) = pipe().unwrap();
    }

    #[test]
    fn fork_child() {
        Fork::new().unwrap();
    }

    #[test]
    fn send_fd() {
        let mut path = std::env::temp_dir();
        path.push("send_fd.txt");

        let mut file = std::fs::File::create(&path).unwrap();
        write!(file, "Hello, World!").unwrap();
        drop(file);

        let mut contents = String::new();

        let uid = unsafe{ libc::getuid() };
        let gid = unsafe{ libc::getgid() };

        match ConnectedFork::with_creds(uid, gid).unwrap() {
            ConnectedFork::Child(mut parent) => {
                let file = std::fs::File::open(&path).unwrap();
                parent.send_fd(file.as_raw_fd()).unwrap();
            },
            ConnectedFork::Parent(_, mut child) => {
                let response = child.recv_fd().unwrap();
                let mut file = unsafe{ std::fs::File::from_raw_fd(response.as_raw_fd()) };
                file.read_to_string(&mut contents).unwrap();
            }
        }

        assert_eq!(&contents, "Hello, World!");
    }
}
