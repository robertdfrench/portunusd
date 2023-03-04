use std::fs;
use std::io;
use std::path;
use errors::define_error_enum;

// Traits
use clap::Parser;
use std::os::fd::FromRawFd;
use std::io::Read;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Override custom door file
    #[arg(short, long, value_name = "FILE")]
    door: Option<path::PathBuf>,
}


#[derive(Debug)]
pub struct ROpenDError {}


define_error_enum!(
    pub enum MainError {
        Io(io::Error),
        Door(doors::Error),
        Utf8(std::string::FromUtf8Error),
        ROpen(ROpenDError)
    }
);

fn main() -> Result<(),MainError> {
    let cli = Cli::parse();
    let door_path = cli.door.unwrap_or(path::Path::new("/var/run/ropen.door").to_path_buf());
    let door_path_str = door_path.to_str().ok_or(io::Error::new(io::ErrorKind::Other, "invalid door path"))?;
    let ropen_client = doors::Client::new(door_path_str)?;
    let (descriptors, error) = ropen_client.call(vec![], b"/home/robert/portunusd/Cargo.toml")?;
    println!("Descriptors: {:?}", descriptors);
    if descriptors.len() == 0 {
        eprintln!("{}", String::from_utf8_lossy(&error));
        Err(ROpenDError{})?;
    }

    let mut cargo_dot_toml = unsafe{ fs::File::from_raw_fd(descriptors[0]) };
    let mut contents = String::new();
    cargo_dot_toml.read_to_string(&mut contents)?;
    println!("Contents of Cargo.toml: {}", contents);

    Ok(())
}

