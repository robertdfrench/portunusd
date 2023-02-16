/* 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2021 Robert D. French
 */
//! Portunus Controller

// Types
use portunusd::door;
use std::io;
use std::path;
use std::process;

// Macros
use portunusd::define_error_enum;

// Traits
use clap::Parser;
use clap::ValueEnum;

define_error_enum!(
    pub enum MainError {
        Door(portunusd::door::Error),
        Utf8(std::string::FromUtf8Error),
        Io(io::Error)
    }
);

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Override custom door file
    #[arg(short, long, value_name = "FILE")]
    door: Option<path::PathBuf>,
    
    /// Override custom porutnusd path
    #[arg(short, long, value_name = "FILE")]
    portunusd: Option<path::PathBuf>,

    #[arg(value_enum)]
    mode: Mode,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    /// Shows whether portunusd is running
    Status,

    /// Start portunusd if not already running
    Start,

    /// Stop portunusd if it is running
    Stop,

    /// Print the version of portunus
    Version
}

fn main() -> Result<(),MainError> {
    let cli = Cli::parse();

    match cli.mode {
        Mode::Status => {
            let door_path = cli.door.unwrap_or(path::Path::new("/var/run/portunusd.door").to_path_buf());

            match door::Client::new(door_path) {
                Ok(portunusd_client) => {
                    let content = portunusd_client.call(vec![], &vec![])?;
                    let response = String::from_utf8(content)?;
                    println!("portunusd is up: {}", response);
                },
                Err(e) => {
                    println!("portunusd is down: {:?}", e);
                }
            }
        },
        Mode::Start => {
            let door_path = cli.door.unwrap_or(path::Path::new("/var/run/portunusd.door").to_path_buf());

            let needs_to_be_started = match door::Client::new(door_path.clone()) {
                Ok(portunusd_client) => !portunusd_client.call(vec![], &vec![]).is_ok(),
                Err(_) => true
            };

            if needs_to_be_started {
                let portunusd_path = cli.portunusd.unwrap_or(path::Path::new("/usr/sbin/portunusd").to_path_buf());
                let portunusd_output = process::Command::new(portunusd_path)
                    .arg("--door")
                    .arg(door_path)
                    .output()?;
                if portunusd_output.status.success() {
                    println!("Started the portunusd server");
                } else {
                    println!("Failed to start the portunusd server");
                }
            } else {
                println!("Portunusd is already running");
            }
        },
        Mode::Stop => {
            let door_path = cli.door.unwrap_or(path::Path::new("/var/run/portunusd.door").to_path_buf());

            match door::Client::new(door_path.clone()) {
                Ok(portunusd_client) => {
                    portunusd_client.call(vec![], &vec![69])?;
                },
                Err(_) => {
                    println!("This thing isn't even running anymore bud");
                }
            }
        }
        Mode::Version => {
            println!("{}", env!("CARGO_PKG_VERSION"));
        }
    }


    Ok(())
}
