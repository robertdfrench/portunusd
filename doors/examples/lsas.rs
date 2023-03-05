use std::io;
use std::path;
use errors::define_error_enum;

// Traits
use clap::Parser;
use std::os::fd::FromRawFd;

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
        Utf8(std::string::FromUtf8Error)
    }
);

fn main() -> Result<(),MainError> {
    let cli = Cli::parse();
    let door_path = cli.door.unwrap_or(path::Path::new("/var/run/lsasd.door").to_path_buf());
    let door_path_str = door_path.to_str().ok_or(io::Error::new(io::ErrorKind::Other, "invalid door path"))?;
    let lsas_client = doors::Client::new(door_path_str)?;
    let (desc, output) = lsas_client.call(vec![], b"alice")?;
    if desc.len() == 0 {
        eprintln!("error: {:?}", output);
        return Ok(());
    }
    let ls_client = unsafe{ doors::Client::from_raw_fd(desc[0]) };
    let (_desc, output) = ls_client.call(vec![], &vec![])?;
    let output = String::from_utf8(output)?;
    println!("Contents of /home/caleb: {}", output);

    Ok(())
}
