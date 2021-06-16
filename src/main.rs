/* 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2021 Robert D. French
 */
//! Portunus Daemon

use portunusd::door;

fn main() -> Result<(),door::Error> {
    let hello_web = door::Client::new("/var/run/hello_web.portunusd")?;
    let greeting = hello_web.call(b"PortunusD")?;
    match String::from_utf8(greeting) {
        Ok(greeting) => println!("{}", greeting),
        Err(_) => panic!("server returned invalid utf-8")
    };
    Ok(())
}
