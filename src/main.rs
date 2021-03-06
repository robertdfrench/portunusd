/* 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2021 Robert D. French
 */
//! Portunus Daemon

use polling::{Event, Poller};
use portunusd::door;
use portunusd::config;
use rayon;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::net::{SocketAddr,TcpListener,TcpStream,UdpSocket};
use std::os::unix::io::{AsRawFd, RawFd};

enum RelaySocket {
    Tcp(TcpListener),
    Udp(UdpSocket),
}

impl AsRawFd for RelaySocket {
    fn as_raw_fd(&self) -> RawFd {
        match self {
            Self::Tcp(socket) => socket.as_raw_fd(),
            Self::Udp(socket) => socket.as_raw_fd(),
        }
    }
}

struct Relay {
    pub socket: RelaySocket,
    pub door: door::Client,
}

#[derive(Debug)]
enum Error {
    Config(config::ParseError),
    IO(std::io::Error)
}

impl From<config::ParseError> for Error {
    fn from(other: config::ParseError) -> Self {
        Self::Config(other)
    }
}

impl From<std::io::Error> for Error {
    fn from(other: std::io::Error) -> Self {
        Self::IO(other)
    }
}

fn read_config(path: &str) -> Result<config::Config,Error> {
    let mut contents = String::new();
    let mut file = File::open(path)?;
    file.read_to_string(&mut contents)?;

    Ok(contents.parse::<config::Config>()?)
}

fn main() -> Result<(),Error> {
    println!("PortunusD {} is booting up!", env!("CARGO_PKG_VERSION"));
    let config = read_config("/opt/local/etc/portunusd.conf")?;

    let mut relays = vec![];
    for statement in &config.statements {
        let door_path = match &statement.target {
            config::ForwardingTarget::Atlas(_) => continue,
            config::ForwardingTarget::Door(door_path) => door_path
        };
        let door = door::Client::new(&door_path).unwrap();
        let socket = match statement.protocol {
            config::Protocol::TCP => {
                let socket = TcpListener::bind(&statement.address).unwrap();
                socket.set_nonblocking(true).unwrap();
                RelaySocket::Tcp(socket)
            },
            config::Protocol::UDP => {
                let socket = UdpSocket::bind(&statement.address).unwrap();
                socket.set_nonblocking(true).unwrap();
                RelaySocket::Udp(socket)
            },
            _ => {
                // treat as generic TCP for now
                let socket = TcpListener::bind(&statement.address).unwrap();
                socket.set_nonblocking(true).unwrap();
                RelaySocket::Tcp(socket)
            }
        };

        relays.push(Relay{ door, socket });
    }

    let poller = Poller::new().unwrap();
    for (key,relay) in relays.iter().enumerate() {
        poller.add(&relay.socket, Event::readable(key)).unwrap();
    }

    let mut events = Vec::new();
    loop {
        events.clear();
        poller.wait(&mut events, None).unwrap();

        for event in &events {
            // A new client has arrived
            let relay = &relays[event.key]; // We know this exists b/c enumerate
            match &relay.socket {
                RelaySocket::Tcp(socket) => {
                    // Okay to unwrap because we know the socket is ready
                    if let Ok((stream, _)) = socket.accept() {
                        let client = relay.door.borrow();
                        rayon::spawn(|| {
                            handle_tcp_stream(stream, client);
                        });
                    }
                },
                RelaySocket::Udp(socket) => {
                    let mut request_buf = [0; 1024];

                    // Okay to unwrap because we know the socket is ready;
                    let (n, addr) = socket.recv_from(&mut request_buf).unwrap();
                    let client = relay.door.borrow();
                    let csocket = socket.try_clone().unwrap();

                    rayon::spawn(move || {
                        let request = &request_buf[..n];
                        handle_udp_socket(request, csocket, addr, client);
                    });
                },
            }

            poller.modify(&relay.socket, Event::readable(event.key)).unwrap();
        }
    }
}

fn handle_udp_socket(request: &[u8], socket: UdpSocket, addr: SocketAddr, client: door::ClientRef) {
    let response = client.call(&request).unwrap();

    let mut offset = 0;
    while offset < response.len() {
        match socket.send_to(&response[offset..], addr) {
            Ok(n) => offset += n,
            Err(e) => {
                eprintln!("error after writing {} bytes to {}: {}", offset, addr, e);
                break;
            }
        }
    }
}

fn handle_tcp_stream(mut stream: TcpStream, client: door::ClientRef) {
    let mut request = [0; 1024];
    match stream.set_nonblocking(false) {
        Ok(_) => {},
        Err(e) => {
            eprintln!("Cancelling request: {}", e);
            return
        }
    }
    match stream.read(&mut request) {
        Ok(_) => {},
        Err(e) => {
            eprintln!("Client hung up: {}", e);
            return
        }
    }

    let response = match client.call(&request) {
        Ok(response) => response,
        Err(e) => {
            eprintln!("Door refused our call: {}", e);
            return
        }
    };

    match stream.write_all(&response) {
        Err(e) => eprintln!("Error responding to client: {}", e),
        Ok(_) => match stream.flush() {
            Err(e) => eprintln!("Could not flush buffered response: {}", e),
            Ok(_) => {},
        },
    }
}
