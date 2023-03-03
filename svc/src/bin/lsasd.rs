use std::fs;
use std::io;
use std::path;
use std::os::fd::RawFd;
use portunusd::doors;
use doors::Server;
use portunusd::derive_server_procedure;
use errors::define_error_enum;

// Traits
use clap::Parser;
use doors::ServerProcedure;
use std::os::fd::IntoRawFd;

fn su_child(username: &str, server: Server) -> ! {
    let uid = match username {
        "robert" => 100,
        "caleb" => 101,
        _ => panic!(),
    };
    unsafe{ libc::setuid(uid); }
    server.park()
}

fn su(_fds: &[RawFd], username: &[u8]) -> (Vec<RawFd>, Vec<u8>) {
    let username = String::from_utf8(username.to_vec()).unwrap();
    let homedir = format!("/home/{}", username);
    let homedir = path::PathBuf::from(homedir);
    let doorpath = homedir.join("ls.door");
    let ls_server = Ls::install(doorpath.to_str().unwrap()).unwrap();
    match unsafe{ libc::fork() } {
        0 => {
            // Child
            su_child(&username, ls_server)
        },
        _child_pid => {
            // Parent
            (vec![ls_server.into_raw_fd()], vec![])
        }
    }
}
derive_server_procedure!(su as Su);

fn ls(_fds: &[RawFd], _data: &[u8]) -> (Vec<RawFd>, Vec<u8>) {
    let mut entries = fs::read_dir(".").unwrap()
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>().unwrap();
    entries.sort();
    let strings = entries.iter()
        .map(|e| e.to_str().ok_or(io::Error::new(io::ErrorKind::Other, "utf8error")))
        .collect::<Result<Vec<&str>, io::Error>>().unwrap();
    let response = strings.join("\n");
    (vec![], response.into_bytes())
}
derive_server_procedure!(ls as Ls);

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
    let door_path = cli.door.unwrap_or(path::Path::new("/var/run/lsasd.door").to_path_buf());
    println!("LsasD is booting up!");
    let door_path_str = door_path.to_str().ok_or(io::Error::new(io::ErrorKind::Other, "invalid door path"))?;
    unsafe{ libc::daemon(0,0) };
    let su_server = Su::install(door_path_str)?;
    su_server.park(); // No return from here
}
