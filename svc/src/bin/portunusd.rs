/* 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2021 Robert D. French
 */
//! Portunus Daemon

// Types
use std::any;
use std::io;
use std::sync::mpsc;
use std::net;
use std::path;
use std::os::fd;
use std::sync::atomic::{AtomicUsize, Ordering};

// Macros
use doors::derive_server_procedure;
use errors::define_error_enum;

// Traits
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Override custom door file
    #[arg(short, long, value_name = "FILE")]
    door: Option<path::PathBuf>,
}

define_error_enum!(
    pub enum MainError {
        Io(io::Error),
        Door(doors::Error),
        Send(mpsc::SendError<net::TcpStream>),
        Join(Box<dyn any::Any + Send>)
    }
);

fn hello(_descriptors: Vec<fd::RawFd>, request: &[u8]) -> (Vec<fd::RawFd>, Vec<u8>) {
    static COUNTER: AtomicUsize = AtomicUsize::new(65);
    if request.len() == 1 {
        if request[0] == 69 {
            std::process::exit(0);
        }
    }

    (vec![], vec![0xF0, 0x9F, 0xA6, 0x80, 32, COUNTER.fetch_add(1, Ordering::Relaxed).try_into().unwrap()])
}
derive_server_procedure!(hello as Hello);

fn main() -> Result<(),MainError> {
    let cli = Cli::parse();
    let door_path = cli.door.unwrap_or(path::Path::new("/var/run/portunusd.door").to_path_buf());
    println!("PortunusD is booting up!");
    let door_path_str = door_path.to_str().ok_or(io::Error::new(io::ErrorKind::Other, "invalid door path"))?;
    unsafe{ libc::daemon(0,0) };
    let hello_server = Hello::install(door_path_str)?;
    hello_server.park(); // No return from here
}
