/* 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2021 Robert D. French
 */
//! Portunus Daemon

use portunusd::door;
use portunusd::plan;
use rayon;
use std::io::{Read,Write};
use std::net::{TcpListener,TcpStream};
use std::thread;
use std::time::Duration;


struct RelayPath {
    pub port: TcpListener,
    pub door: door::Client,
}

fn main() {
    let relay_statements = vec![
        "forward 0.0.0.0:80 to /var/run/hello_web.portunusd",
        "forward 0.0.0.0:8080 to /var/run/go_away.portunusd",
    ];


    let mut paths = vec![];
    for statement in &relay_statements {
        let plan: plan::RelayPlan = statement.parse().unwrap();
        paths.push(RelayPath{
            door: door::Client::new(&plan.application_path.to_str().unwrap()).unwrap(),
            port: TcpListener::bind(&plan.network_address).unwrap(),
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
