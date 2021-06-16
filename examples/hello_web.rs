/* 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2021 Robert D. French
 */
use portunusd::derive_server_procedure;
use portunusd::door;
use std::fmt::format;
use std::str::from_utf8;

// Consider the function `hello`, which returns a polite greeting to a client:
pub fn hello(request: &[u8]) -> Vec<u8> {
    match from_utf8(request) {
        Err(_) => b"I couldn't understand your name!".to_vec(),
        Ok(name) => {
            let response = format!("Hello, {}!", name);
            response.into_bytes()
        }
    }
}

derive_server_procedure!(hello as Hello);

fn main() {
    println!("Booting HelloWeb Application");
    let _hello_server = Hello::install("/var/run/hello_web.portunusd").unwrap();
    loop {
        std::thread::yield_now();
    }
}
