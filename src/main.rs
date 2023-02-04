/* 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2021 Robert D. French
 */
//! Portunus Daemon

// Types
use portunusd::counter;
use portunusd::door;
use portunusd::derive_server_procedure;
use std::any;
use std::io;
use std::sync::mpsc;
use std::net;
use std::thread;
use std::os::fd;

// Macros
#[macro_use]
mod errors;

// Traits
use std::io::Write;
use std::os::fd::FromRawFd;
use std::os::fd::IntoRawFd;

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

fn hello(descriptors: Vec<fd::RawFd>, _request: &[u8]) -> (Vec<fd::RawFd>, Vec<u8>) {
    if descriptors.len() > 0 {
        let mut client = unsafe{ std::net::TcpStream::from_raw_fd(descriptors[0]) };
        writeln!(&mut client, "HTTP/1.1 200 OK").unwrap();
        writeln!(&mut client, "Content-Type: text/plain").unwrap();
        writeln!(&mut client, "Content-Length: 6").unwrap();
        writeln!(&mut client, "").unwrap();
        writeln!(&mut client, "Hello").unwrap();
    }
    (vec![], vec![])
}
derive_server_procedure!(hello as Hello);

fn main() -> Result<(),MainError> {
    for arg in std::env::args() {
        if arg == "hello" {
            println!("Hello App is booting up!");
            let hello_server = Hello::install("hello.door")?;
            hello_server.park(); // No return from here
        }
    }
    println!("PortunusD {} is booting up!", env!("CARGO_PKG_VERSION"));
    let hello_client = door::Client::new("hello.door")?;

    let listener = net::TcpListener::bind("0.0.0.0:8080")?;
    let mut door_attendants = vec![];
    let num_cpus = thread::available_parallelism()?;

    for _ in 0..num_cpus.get() {
        door_attendants.push(DoorAttendant::new(hello_client.borrow()));
    }

    let mut rr = counter::RoundRobin::new(&door_attendants);
    for stream in listener.incoming() {
        let stream = stream?;

        rr.next().send(stream)?;
    }

    for attendant in door_attendants {
        attendant.join()?;
    }

    Ok(())
}
