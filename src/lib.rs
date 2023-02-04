/* 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2021 Robert D. French
 */
//! [Crate](https://crates.io/crates/portunusd)        &VerticalSeparator;
//! [Repo](https://github.com/robertdfrench/portunusd) &VerticalSeparator;
//! [Tweets](https://twitter.com/portunusd)
//!
//! PortunusD is a network application server, inspired by relayd and inetd, which aims to ease the
//! scaling of single-threaded request/response-style applications: web applications, DNS queries,
//! etc. PortunusD allows applications to embrace the "serverless" style of development, but without
//! throwing away all the luxuries of the operating system.


pub mod config;
pub mod counter;
pub mod door;
pub mod errors;
pub mod illumos;
