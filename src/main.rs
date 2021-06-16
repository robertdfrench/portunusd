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
use std::net::{TcpListener,TcpStream};
use std::io::{Read,Write};

fn main() {
    let hello_web = door::Client::new("/var/run/hello_web.portunusd").unwrap();
    let listener = TcpListener::bind("0.0.0.0:80").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let client = hello_web.borrow();

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
