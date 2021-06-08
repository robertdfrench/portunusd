/* 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2021 Robert D. French
 */

//! Portunus - The God of Ports and Doors
//!
//! Portunus is a network application server, inspired by relayd and inetd, which aims to ease the
//! scaling of single-threaded request/response-style applications: web applications, DNS queries,
//! etc. Portunus allows applications to embrace the "serverless" style of development, but without
//! throwing away all the luxuries of the operating system.

pub mod illumos;
pub mod jamb;
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
