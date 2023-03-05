use std::env;
use std::fs;
use std::io;
use std::net;
use std::path;
use std::os::unix::net::UnixStream;
use std::os::fd::RawFd;
use doors::derive_server_procedure;
use errors::define_error_enum;

// Traits
use clap::Parser;
use doors::ServerProcedure;
use std::os::fd::IntoRawFd;
use io::Read;
use io::Write;

fn su(_fds: &[RawFd], username: &[u8]) -> (Vec<RawFd>, Vec<u8>) {
    let (mut parentsock, mut childsock) = match UnixStream::pair() {
        Ok((parentsock, childsock)) => (parentsock, childsock),
        Err(e) => {
            eprintln!("Couldn't create a pair of sockets: {e:?}");
            return (vec![], vec![]);
        }
    };
    parentsock.shutdown(net::Shutdown::Write).unwrap();
    childsock.shutdown(net::Shutdown::Read).unwrap();
    match unsafe{ libc::fork() } {
        0 => {
            // Child
            // let parentfd = parentsock.into_raw_fd();
            // unsafe{ libc::close(parentfd) };
            let username = String::from_utf8(username.to_vec()).unwrap();
            let uid = match username.as_str() {
                "alice" => 102,
                "bob" => 103,
                _ => panic!(),
            };
            unsafe{ libc::setuid(uid); }
            let homedir = format!("/home/{}", username);
            let homedir = path::PathBuf::from(homedir);
            env::set_current_dir(&homedir).unwrap();
            println!("About to install a door in {:?}", homedir);
            let ls_server = Ls::install("ls.door").unwrap();
            write!(childsock, "{}", homedir.join("ls.door").display()).unwrap();
            parentsock.shutdown(net::Shutdown::Read).unwrap();
            // TODO: This depends very much on my personal workstation
            ls_server.park()
        },
        _child_pid => {
            // Parent
            // let childfd = childsock.into_raw_fd();
            // unsafe{ libc::close(childfd) };
            let mut door_path = String::new();
            parentsock.read_to_string(&mut door_path).unwrap();
            println!("I want to open {}", door_path);
            let _door_client = doors::Client::new(&door_path).unwrap();
            //let fds = vec![door_client.into_raw_fd()];
            (vec![], vec![])
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
    // unsafe{ libc::daemon(0,0) };
    let su_server = Su::install(door_path_str)?;
    su_server.park(); // No return from here
}
