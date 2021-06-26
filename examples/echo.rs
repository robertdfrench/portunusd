/* 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2021 Robert D. French
 */
//! An [RFC862] compatible UDP Echo Server
//!
//! Packets go in, packets go out. Can't explain that.
//!
//! [RFC862]: https://datatracker.ietf.org/doc/html/rfc862
use portunusd::derive_server_procedure;


/// The Server Procedure
///
/// This function is the guts of the echo service. It (or really, a function derived from it) is
/// made available on the filesystem as a Door. Its only job is to return the bytes that it is
/// given as input.
pub fn echo(request: &[u8]) -> Vec<u8> {
    request.to_vec()
}


// Define a type `Echo` which has, among other things, a libc-compatible wrapper for the `echo`
// server procedure defined above. This wrapper is later passed off to the Doors API, and the other
// convenience functions defined on this type make it easier to write a `main()` function which
// sets up our echo service.
derive_server_procedure!(echo as Echo);


fn main() {
    println!("Booting Echo Application");
    // Install the server procedure on the filesystem
    let echo_server = Echo::install("/var/run/echo.portunusd").unwrap();
    // Put this (main) thread back into the door pool
    echo_server.park();
}
