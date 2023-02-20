/* 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2021 Robert D. French
 */
//! Portunus Daemon

// Types
use portunusd::door;
use std::any;
use std::io;
use std::sync::mpsc;
use std::net;
use std::path;
use std::thread;
use std::os::fd;

// Macros
use portunusd::derive_server_procedure;
use errors::define_error_enum;

// Traits
use clap::Parser;
use std::os::fd::IntoRawFd;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Override custom door file
    #[arg(short, long, value_name = "FILE")]
    door: Option<path::PathBuf>,
}

define_error_enum!(
    pub enum MainError {
        Io(io::Error),
        Door(portunusd::door::Error),
        Send(mpsc::SendError<net::TcpStream>),
        Join(Box<dyn any::Any + Send>)
    }
);

define_error_enum!(
    pub enum AttendError {
        Io(io::Error),
        Recv(mpsc::RecvError),
        Door(portunusd::door::Error)
    }
);

struct DoorAttendant {
    sender: mpsc::Sender<net::TcpStream>,
    join_handle: thread::JoinHandle<()>
}

impl DoorAttendant {
    fn new(doorc: door::ClientRef) -> Self {
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

    fn attend(receiver: &mut mpsc::Receiver<net::TcpStream>, doorc: door::ClientRef) -> Result<(), AttendError> {
        let client = receiver.recv()?;
        doorc.call(vec![client.into_raw_fd()], &vec![])?;
        Ok(())
    }

    fn send(&self, stream: net::TcpStream) -> Result<(), mpsc::SendError<net::TcpStream>> {
        self.sender.send(stream)
    }

    fn join(self) -> Result<(), Box<(dyn any::Any + Send + 'static)>> {
        self.join_handle.join()
    }
}

fn hello(_descriptors: Vec<fd::RawFd>, request: &[u8]) -> (Vec<fd::RawFd>, Vec<u8>) {
    if request.len() == 1 {
        if request[0] == 69 {
            std::process::exit(0);
        }
    }
    (vec![], vec![0xF0, 0x9F, 0xA6, 0x80])
}
derive_server_procedure!(hello as Hello);

fn main() -> Result<(),MainError> {
    let cli = Cli::parse();
    let door_path = cli.door.unwrap_or(path::Path::new("/var/run/portunusd.door").to_path_buf());
    println!("PortunusD is booting up!");
    let door_path_str = door_path.to_str().ok_or(io::Error::new(io::ErrorKind::Other, "invalid door path"))?;
    unsafe{ libc::daemon(0,0) };
    let hello_server = Hello::install(door_path_str)?;
    hello_server.park(); // No return from here
}
