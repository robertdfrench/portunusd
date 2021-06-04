extern crate portunus;

use portunus::relay_plan::RelayPlan;
use std::net::{TcpListener, TcpStream};
use std::os::unix::io::{IntoRawFd, RawFd};

struct RelayLink {
    door: RawFd,
    port: RawFd,
}

fn main() {
    let statements = [
        "forward 0.0.0.0 port 8080 to /var/run/hello_web_door",
        "forward 0.0.0.0 port 1234 to /var/run/caasio_door"
    ];
    let _plans = statements.iter().filter_map(|statement| {
        statement.parse::<RelayPlan>().ok()
    });

    println!("Hello, world!");
}
