use std::env;
use std::fs;
use std::io;
use std::path;
use std::os::fd::RawFd;
use connected_fork::ConnectedFork;
use doors::derive_server_procedure;
use errors::define_error_enum;

// Traits
use clap::Parser;
use doors::ServerProcedure;
use std::os::fd::AsRawFd;

static mut USER_DOORS: &'static mut [RawFd] = &mut [-1; 1024];

fn su(_fds: &[RawFd], username: &[u8]) -> (Vec<RawFd>, Vec<u8>) {
    let username = String::from_utf8(username.to_vec()).unwrap();
    let uid = match username.as_str() {
        "alice" => 102,
        "bob" => 103,
        _ => panic!(),
    };
    if unsafe{ USER_DOORS[uid] } == -1 {
        match ConnectedFork::with_creds(uid as libc::uid_t, uid as libc::uid_t).unwrap() {
            ConnectedFork::Child(mut parent) => {
                // Child
                let homedir = format!("/home/{}", username);
                let homedir = path::PathBuf::from(homedir);
                env::set_current_dir(&homedir).unwrap();
                println!("About to install a door in {:?}", homedir);
                let ls_server = Ls::install("ls.door").unwrap();
                parent.send_fd(ls_server.door_descriptor.as_raw_fd()).unwrap();
                ls_server.park()
            },
            ConnectedFork::Parent(_pid, mut child) => {
                // Parent
                let creds = child.recv_fd().unwrap();
                let fd = creds.as_raw_fd();
                unsafe{ USER_DOORS[uid] = fd };
                let fd2 = unsafe{ libc::dup(fd) };
                (vec![fd2], vec![])
            }
        }
    } else {
        let fd = unsafe{ USER_DOORS[uid] };
        let fd2 = unsafe{ libc::dup(fd) };
        (vec![fd2], vec![])
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
    unsafe{ libc::daemon(1,1) };
    let su_server = Su::install(door_path_str)?;
    su_server.park(); // No return from here
}
