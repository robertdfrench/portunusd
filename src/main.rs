/* 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2021 Robert D. French
 */
//! Portunus Daemon

use portunusd::door;
use rayon;
use std::io::{Read,Write};
use std::net::{TcpListener,TcpStream};
use std::thread;
use std::time::Duration;

struct RelayPlan {
    network: String,
    application: String,
}

struct RelayPath {
    pub port: TcpListener,
    pub door: door::Client,
}


fn main() {
    let plans = vec![
        RelayPlan{
            application: "/var/run/hello_web.portunusd".to_owned(),
            network: "0.0.0.0:80".to_owned()
        },
        RelayPlan{
            application: "/var/run/go_away.portunusd".to_owned(),
            network: "0.0.0.0:8080".to_owned()
        },
    ];

    let mut paths = vec![];
    for plan in &plans {
        paths.push(RelayPath{
            door: door::Client::new(&plan.application).unwrap(),
            port: TcpListener::bind(&plan.network).unwrap(),
        });
    }


    while let Some(path) = paths.pop() {
        rayon::spawn(|| {
            monitor(path);
        });
    }

    thread::sleep(Duration::new(u64::MAX, 1_000_000_000 - 1));
}

fn monitor(path: RelayPath) {
    for stream in path.port.incoming() {
        let stream = stream.unwrap();
        let client = path.door.borrow();

        rayon::spawn(|| {
            handle_connection(stream, client);
        });
    }
}


fn handle_connection(mut stream: TcpStream, client: door::ClientRef) {
    let mut request = [0; 1024];
    stream.read(&mut request).unwrap();

    let response = client.call(&request).unwrap();

    stream.write(&response).unwrap();
    stream.flush().unwrap();
}
