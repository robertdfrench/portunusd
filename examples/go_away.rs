/* 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2021 Robert D. French
 */
use portunusd::derive_server_procedure;
use std::str::from_utf8;

fn content_type() -> String {
    "Content-Type: text/html; charset=UTF-8".to_owned()
}

fn content_length(msg: &str) -> String {
    let len = msg.to_owned().into_bytes().len();
    format!("Content-Length: {}", len)
}

fn four_hundred(msg: &str) -> String {
    format!("HTTP/1.1 400 Bad Request\n{}\n{}\n\n{}",
        content_type(),
        content_length(msg),
        msg
    )
}

fn two_hundred(msg: &str) -> String {
    format!("HTTP/1.1 200 OK\n{}\n{}\n\n{}",
        content_type(),
        content_length(msg),
        msg
    )
}

fn handle_http_request(request: &str) -> String {
    match request.lines().nth(0) {
        None => four_hundred("Your http request was empty, or pretty close anyhow"),
        Some(request_line) => {
            let elements: Vec<&str> = request_line.split(' ').collect();
            if elements.len() != 3 {
                return four_hundred("Your http request line is sorta goofy");
            }
            if elements[0] != "GET" {
                return four_hundred("I only accept GET requests");
            }
            let path = elements[1];
            let components: Vec<&str> = path.split('/').collect();
            match components.last() {
                None => four_hundred("There really should be a last component here"),
                Some(name) => two_hundred(&format!("Go away, {}!", name)),
            }
        }
    }
}

// Consider the function `hello`, which returns a polite greeting to a client:
pub fn hello(request: &[u8]) -> Vec<u8> {
    let response = match from_utf8(request) {
        Err(_) => four_hundred("I couldn't understand your name!"),
        Ok(request) => handle_http_request(request),
    };
    response.into_bytes()
}

derive_server_procedure!(hello as Hello);

fn main() {
    println!("Booting GoAway Application");
    let _hello_server = Hello::install("/var/run/go_away.portunusd").unwrap();
    loop {
        std::thread::yield_now();
    }
}
