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
use portunusd::plan;
use rayon;
use std::io::{Read,Write};
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
    Config(plan::ParseError)
}

impl From<plan::ParseError> for Error {
    fn from(other: plan::ParseError) -> Self {
        Self::Config(other)
    }
}

fn main() -> Result<(),Error> {
    println!("PortunusD {} is booting up!", env!("CARGO_PKG_VERSION"));
    let plans = plan::parse_config("/opt/local/etc/portunusd.conf")?;

    let mut relays = vec![];
    for plan in &plans {
        let door = door::Client::new(&plan.application_path).unwrap();
        let socket = match plan.protocol {
            plan::RelayProtocol::Tcp => {
                let socket = TcpListener::bind(&plan.network_address).unwrap();
                socket.set_nonblocking(true).unwrap();
                RelaySocket::Tcp(socket)
            },
            plan::RelayProtocol::Udp => {
                let socket = UdpSocket::bind(&plan.network_address).unwrap();
                socket.set_nonblocking(true).unwrap();
                RelaySocket::Udp(socket)
            },
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
    stream.set_nonblocking(false).unwrap();
    match stream.read(&mut request) {
        Ok(_) => {},
        Err(e) => {
            eprintln!("Client hung up: {}", e);
            return
        }
    }

    let response = client.call(&request).unwrap();

    match stream.write_all(&response) {
        Err(e) => eprintln!("Error responding to client: {}", e),
        Ok(_) => match stream.flush() {
            Err(e) => eprintln!("Could not flush buffered response: {}", e),
            Ok(_) => {},
        },
    }
}
