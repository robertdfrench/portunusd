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
use std::net::{TcpListener,TcpStream};


struct Relay {
    pub port: TcpListener,
    pub door: door::Client,
}

fn main() {
    let relay_statements = vec![
        "forward 0.0.0.0:80 to /var/run/hello_web.portunusd",
        "forward 0.0.0.0:8080 to /var/run/go_away.portunusd",
    ];


    let mut relays = vec![];
    for statement in &relay_statements {
        let plan: plan::RelayPlan = statement.parse().unwrap();
        relays.push(Relay{
            door: door::Client::new(&plan.application_path.to_str().unwrap()).unwrap(),
            port: TcpListener::bind(&plan.network_address).unwrap(),
        });
    }

    let poller = Poller::new().unwrap();
    for (key,relay) in relays.iter().enumerate() {
        relay.port.set_nonblocking(true).unwrap();
        poller.add(&relay.port, Event::readable(key)).unwrap();
    }

    let mut events = Vec::new();
    loop {
        events.clear();
        poller.wait(&mut events, None).unwrap();

        for event in &events {
            // A new client has arrived
            let relay = &relays[event.key]; // We know this exists b/c enumerate
            let (stream, _) = relay.port.accept().unwrap();

            let client = relay.door.borrow();
            rayon::spawn(|| {
                handle_connection(stream, client);
            });

            poller.modify(&relay.port, Event::readable(event.key)).unwrap();
        }
    }
}

fn handle_connection(mut stream: TcpStream, client: door::ClientRef) {
    let mut request = [0; 1024];
    stream.set_nonblocking(false).unwrap();
    stream.read(&mut request).unwrap();

    let response = client.call(&request).unwrap();

    stream.write(&response).unwrap();
    stream.flush().unwrap();
}
