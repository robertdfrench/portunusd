/* 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2021 Robert D. French
 */
//! Configuration commands for PortunusD
//!
//! PortunusD's only job is to relay network traffic to applications. As such, its configuration is
//! limited to statements about what kind of relay situations it should expect to find itself in.
//! These statements, known as RelayPlans, take the following form:
//!
//! ```portunusd.conf
//! forward tcp 0.0.0.0:80 to /var/run/hello_web.door
//! forward tcp 0.0.0.0:81 to /var/run/go_away.door
//! forward udp 0.0.0.0:7 to /var/run/go_away.door
//! ```


use std::fs::File;
use std::io::Read;
use std::net::AddrParseError;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;


/// Parse relay plans from a config file
pub fn parse_config(path: &str) -> Result<Vec<RelayPlan>,ParseError> {
    let mut contents = String::new();
    let mut f = File::open(path)?;
    f.read_to_string(&mut contents)?;

    let mut plans = vec![];
    for line in contents.lines() {
        let plan: RelayPlan = line.parse()?;
        plans.push(plan);
    }

    Ok(plans)
}

/// An individual forwarding statement.
///
/// A RelayPlan has one client-facing network address and one application-facing door path. Whether
/// or not these addresses exist is a problem for later in the process; the RelayPlan only cares
/// about parsing this information from a configuration string.
///
/// ```
/// use portunusd::plan::RelayPlan;
/// use portunusd::plan::RelayProtocol;
///
/// let plan: RelayPlan = "forward tcp 1.2.3.4:5678 to /a/b/c.door".parse().unwrap();
/// assert!(plan.network_address.is_ipv4());
/// assert_eq!(plan.network_address.port(), 5678);
/// assert_eq!(plan.protocol, RelayProtocol::Tcp);
/// assert_eq!(plan.application_path.extension().unwrap(), "door");
/// ```
pub struct RelayPlan {
    /// Network ingress from client
    pub network_address: SocketAddr,
    /// Door egress to application
    pub application_path: PathBuf,
    /// How should we listen to the socket?
    pub protocol: RelayProtocol,
}

#[derive(Debug,PartialEq)]
pub enum RelayProtocol {
    Tcp,
    Udp,
}

/// Recursive-descent parser element for RelayProtocols.
impl FromStr for RelayProtocol {
    type Err = ParseError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        match source {
            "tcp" => Ok(RelayProtocol::Tcp),
            "udp" => Ok(RelayProtocol::Udp),
            _ => Err(Self::Err::Syntax)
        }
    }
}

/// Problems that can arise when parsing a RelayPlan
///
/// The `Syntax` variant is a catch-all any situation that doesn't yield *something* for either the
/// network address or the application's door path. There is, unfortunately, no a-priori invalid
/// way to specify a path in UNIX, so any string in the fourth position will be acceptable. We
/// defer to Rust's `SocketAddr` and its associated `AddrParseError` to make sense of the string in
/// the second position.
#[derive(Debug)]
pub enum ParseError {
    /// The 2nd string in the statement could not be parsed as a network address.
    Network(AddrParseError),
    /// A file opening / reading error occurred
    Io(std::io::Error),
    /// The statement did not follow the `forward {addr} to {path}` syntax.
    Syntax,
}


/// Wrap Rust's `AddrParseError` in our own type.
impl From<AddrParseError> for ParseError {
    fn from(other: AddrParseError) -> Self {
        Self::Network(other)
    }
}


/// Wrap IO Error in our own type
impl From<std::io::Error> for ParseError {
    fn from(other: std::io::Error) -> Self {
        Self::Io(other)
    }
}


/// Recursive-descent parser element for RelayPlan statements.
impl FromStr for RelayPlan {
    type Err = ParseError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        let mut components = source.split_ascii_whitespace();
        match components.next() {
            Some("forward") => {},
            _ => return Err(Self::Err::Syntax)
        }

        let protocol = match components.next() {
            Some(protocol) => protocol.parse::<RelayProtocol>()?,
            None => return Err(Self::Err::Syntax)
        };

        let network_address = match components.next() {
            Some(address) => address.parse::<SocketAddr>()?,
            None => return Err(Self::Err::Syntax)
        };

        match components.next() {
            Some("to") => {},
            _ => return Err(Self::Err::Syntax)
        }

        let application_path = match components.next() {
            Some(path) => path.parse::<PathBuf>().unwrap(), // this is infallible
            None => return Err(Self::Err::Syntax)
        };

        Ok(Self{ network_address, application_path, protocol })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::net::IpAddr;
    use std::net::Ipv4Addr;
    use std::net::Ipv6Addr;

    #[test]
    fn can_parse() {
        let plan: RelayPlan = "forward tcp 0.0.0.0:80 to /var/run/hello_web.portunusd".parse().unwrap();
        assert!(plan.network_address.is_ipv4());
        assert_eq!(plan.network_address.port(), 80);
        assert_eq!(plan.protocol, RelayProtocol::Tcp);
        assert_eq!(plan.network_address.ip(), IpAddr::V4(Ipv4Addr::new(0,0,0,0)));
        assert_eq!(plan.application_path.to_str(), Some("/var/run/hello_web.portunusd"));
    }

    #[test]
    fn supports_ipv6() {
        let plan: RelayPlan = "forward tcp [2001:db8::1]:8080 to /door/path".parse().unwrap();
        assert!(plan.network_address.is_ipv6());
        assert_eq!(plan.network_address.port(), 8080);
        let addr = Ipv6Addr::new(0x2001,0xdb8,0,0,0,0,0,0x1);
        assert_eq!(plan.network_address.ip(), IpAddr::V6(addr));
    }

    #[test]
    #[should_panic]
    fn missing_door_path() {
        let _plan: RelayPlan = "forward 0.0.0.0:80 to".parse().unwrap();
    }

    #[test]
    #[should_panic]
    fn typo() {
        "foward tcp 0.0.0.0:80 to /var/run/hello_web.portunusd".parse::<RelayPlan>().unwrap();
    }

    #[test]
    fn parse_udp() {
        let protocol: RelayProtocol = "udp".parse().unwrap();
        assert_eq!(protocol, RelayProtocol::Udp);
    }

    #[test]
    fn parse_tcp() {
        let protocol: RelayProtocol = "tcp".parse().unwrap();
        assert_eq!(protocol, RelayProtocol::Tcp);
    }

    #[test]
    #[should_panic]
    fn parse_protocol_error() {
        // SCTP isn't supported, so it should trigger an error
        "sctp".parse::<RelayProtocol>().unwrap();
    }
}
