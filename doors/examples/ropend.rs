use std::io;
use std::fs::File;
use std::path;
use std::os::fd::RawFd;

use doors::derive_server_procedure;
use errors::define_error_enum;

// Traits
use clap::Parser;
use doors::ServerProcedure;
use std::os::fd::IntoRawFd;

fn open(_fds: &[RawFd], data: &[u8]) -> (Vec<RawFd>, Vec<u8>) {
    match String::from_utf8(data.to_vec()) {
        Err(e) => (vec![], format!("ROpenD: {:?}", e).into_bytes()),
        Ok(file_path) => {
            println!("About to open: {}", file_path);
            match File::open(&file_path) {
                Err(e) => (vec![], format!("ROpenD: {:?}", e).into_bytes()),
                Ok(f) => {
                    let raw = f.into_raw_fd();
                    println!("Descriptor number: {}", raw);
                    (vec![raw], vec![])
                }
            }
        }
    }
}
derive_server_procedure!(open as Open);

define_error_enum!(
    pub enum MainError {
        Io(io::Error),
        Door(doors::Error)
    }
);

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Override custom door file
    #[arg(short, long, value_name = "FILE")]
    door: Option<path::PathBuf>,
}

fn main() -> Result<(),MainError> {
    let cli = Cli::parse();
    let door_path = cli.door.unwrap_or(path::Path::new("/var/run/ropend.door").to_path_buf());
    println!("ROpenD is booting up!");
    let door_path_str = door_path.to_str().ok_or(io::Error::new(io::ErrorKind::Other, "invalid door path"))?;
    // unsafe{ libc::daemon(0,0) };
    let open_server = Open::install(door_path_str)?;
    open_server.park(); // No return from here
}
