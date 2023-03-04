/* 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2023 Robert D. French
 */

// Types
use std::any;
use std::io;
use std::sync::mpsc;
use std::net;
use std::thread;

// Macros
use errors::define_error_enum;

// Traits
use std::os::fd::IntoRawFd;

define_error_enum!(
    pub enum AttendError {
        Io(io::Error),
        Recv(mpsc::RecvError),
        Door(doors::Error)
    }
);

pub struct DoorAttendant {
    pub sender: mpsc::Sender<net::TcpStream>,
    pub join_handle: thread::JoinHandle<()>
}

impl DoorAttendant {
    pub fn new(doorc: doors::ClientRef) -> Self {
        let (sender, mut receiver) = mpsc::channel();
        let join_handle = thread::spawn(move|| {
            loop {
                if let Err(e) = Self::attend(&mut receiver, doorc) {
                    eprintln!("Door error: {:?}", e);
                    let name = std::ffi::CString::new("Door problem").expect("CString::new failed");
                    unsafe{ libc::perror(name.as_ptr()) };
                }
            }
        });
        Self{ sender, join_handle }
    }

    pub fn attend(receiver: &mut mpsc::Receiver<net::TcpStream>, doorc: doors::ClientRef) -> Result<(), AttendError> {
        let client = receiver.recv()?;
        doorc.call(vec![client.into_raw_fd()], &vec![])?;
        Ok(())
    }

    pub fn send(&self, stream: net::TcpStream) -> Result<(), mpsc::SendError<net::TcpStream>> {
        self.sender.send(stream)
    }

    pub fn join(self) -> Result<(), Box<(dyn any::Any + Send + 'static)>> {
        self.join_handle.join()
    }
}
