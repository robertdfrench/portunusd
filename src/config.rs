/* 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2021 Robert D. French
 */


use std::convert::From;
use std::convert::Infallible;
use std::net::AddrParseError;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;


/// HTTP Request Methods
///
/// See https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods
#[derive(Debug,PartialEq)]
pub enum Method {
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    CONNECT,
    OPTIONS,
    TRACE,
    PATCH
}

#[derive(Debug,PartialEq)]
pub enum ParseMethodError {
    UnrecognizedMethod
}

impl FromStr for Method {
    type Err = ParseMethodError;

    fn from_str(input: &str) -> Result<Self,Self::Err> {
        match input {
            "GET" => Ok(Self::GET),
            "HEAD" => Ok(Self::HEAD),
            "POST" => Ok(Self::POST),
            "PUT" => Ok(Self::PUT),
            "DELETE" => Ok(Self::DELETE),
            "CONNECT" => Ok(Self::CONNECT),
            "OPTIONS" => Ok(Self::OPTIONS),
            "TRACE" => Ok(Self::TRACE),
            "PATCH" => Ok(Self::PATCH),
            _ => Err(Self::Err::UnrecognizedMethod)
        }
    }
}


#[derive(Debug,PartialEq)]
pub struct MapStatement {
    method: Method,
    prefix: PathBuf,
    door: PathBuf
}

#[derive(Debug,PartialEq)]
pub enum ParseMapStatementError {
    Method(ParseMethodError),
    Syntax
}

impl From<ParseMethodError> for ParseMapStatementError {
    fn from(other: ParseMethodError) -> Self {
        Self::Method(other)
    }
}

impl From<Infallible> for ParseMapStatementError {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

impl FromStr for MapStatement {
    type Err = ParseMapStatementError ;

    /// map GET /primes to /var/run/eratosthenes.door
    fn from_str(input: &str) -> Result<Self,Self::Err> {
        let mut parts = input.split_whitespace();

        if parts.next() != Some("map") {
            return Err(Self::Err::Syntax);
        }

        let method: Method = match parts.next() {
            Some(m) => m.parse()?,
            None => return Err(Self::Err::Syntax)
        };

        let prefix: PathBuf = match parts.next() {
            Some(p) => p.parse()?,
            None => return Err(Self::Err::Syntax)
        };

        if parts.next() != Some("to") {
            return Err(Self::Err::Syntax);
        }

        let door: PathBuf = match parts.next() {
            Some(d) => d.parse()?,
            None => return Err(Self::Err::Syntax)
        };

        Ok(Self{ method, prefix, door })
    }
}

#[derive(Debug,PartialEq)]
pub struct Atlas {
    maps: Vec<MapStatement>
}

#[derive(Debug,PartialEq)]
pub enum ParseAtlasError {
    Map(ParseMapStatementError),
    Syntax
}

impl From<ParseMapStatementError> for ParseAtlasError {
    fn from(other: ParseMapStatementError) -> Self {
        Self::Map(other)
    }
}

impl FromStr for Atlas {
    type Err = ParseAtlasError ;

    /// map GET /primes to /var/run/eratosthenes.door
    fn from_str(input: &str) -> Result<Self,Self::Err> {
        let mut parts = input.split_whitespace();

        if parts.next() != Some("{") {
            return Err(Self::Err::Syntax);
        }

        if parts.nth_back(0) != Some("}") {
            return Err(Self::Err::Syntax);
        }

        let mut maps: Vec<MapStatement> = vec![];

        let loop_parts = &mut parts;

        loop {
            let map_statement = loop_parts.take(5);
            let map_statement: Vec<&str> = map_statement.collect();
            let map_statement = map_statement.join(" ");
            let map_statement = match map_statement.as_str() {
                "" => break,
                _ => map_statement.parse::<MapStatement>()?
            };
            maps.push(map_statement)
        }

        Ok(Self{ maps })
    }
}


#[derive(Debug,PartialEq)]
pub enum ForwardingTarget {
    Door(PathBuf),
    Atlas(Atlas)
}

#[derive(Debug,PartialEq)]
pub enum ParseForwardingTargetError {
    Atlas(ParseAtlasError),
    Syntax
}

impl From<Infallible> for ParseForwardingTargetError {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

impl From<ParseAtlasError> for ParseForwardingTargetError {
    fn from(other: ParseAtlasError) -> Self {
        Self::Atlas(other)
    }
}

impl FromStr for ForwardingTarget {
    type Err = ParseForwardingTargetError;

    fn from_str(input: &str) -> Result<Self,Self::Err> {
        if input.starts_with("/") {
            let door: PathBuf = input.parse()?;
            return Ok(Self::Door(door))
        } else if input.starts_with("{") {
            let atlas: Atlas = input.parse()?;
            return Ok(Self::Atlas(atlas))
        } else {
            return Err(Self::Err::Syntax)
        }
    }
}

#[derive(Debug,PartialEq)]
pub enum Protocol {
    UDP,
    TCP,
    TLS,
    HTTP,
    HTTPS
}

#[derive(Debug,PartialEq)]
pub enum ParseProtocolError {
    UnrecognizedProtocol
}

impl FromStr for Protocol {
    type Err = ParseProtocolError;

    fn from_str(input: &str) -> Result<Self,Self::Err> {
        match input {
            "udp" => Ok(Self::UDP),
            "tcp" => Ok(Self::TCP),
            "tls" => Ok(Self::TLS),
            "http" => Ok(Self::HTTP),
            "https" => Ok(Self::HTTPS),
            _ => Err(Self::Err::UnrecognizedProtocol)
        }
    }
}

#[derive(Debug,PartialEq)]
pub struct ForwardingStatement {
    protocol: Protocol,
    address: SocketAddr,
    target: ForwardingTarget
}

#[derive(Debug,PartialEq)]
pub enum ParseForwardingStatementError {
    Protocol(ParseProtocolError),
    Address(AddrParseError),
    Target(ParseForwardingTargetError),
    Syntax
}

impl From<ParseProtocolError> for ParseForwardingStatementError {
    fn from(other: ParseProtocolError) -> Self {
        Self::Protocol(other)
    }
}

impl From<AddrParseError> for ParseForwardingStatementError {
    fn from(other: AddrParseError) -> Self {
        Self::Address(other)
    }
}

impl From<ParseForwardingTargetError> for ParseForwardingStatementError {
    fn from(other: ParseForwardingTargetError) -> Self {
        Self::Target(other)
    }
}

impl FromStr for ForwardingStatement {
    type Err = ParseForwardingStatementError;

    fn from_str(input: &str) -> Result<Self,Self::Err> {
        let mut parts = input.split_whitespace();

        if parts.next() != Some("forward") {
            return Err(Self::Err::Syntax);
        }

        let protocol: Protocol = match parts.next() {
            Some(p) => p.parse()?,
            None => return Err(Self::Err::Syntax)
        };

        let address: SocketAddr = match parts.next() {
            Some(a) => a.parse()?,
            None => return Err(Self::Err::Syntax)
        };

        if parts.next() != Some("to") {
            return Err(Self::Err::Syntax);
        }

        let target: ForwardingTarget = match parts.next() {
            Some("{") => {
                let atlas: Vec<&str> = parts.take_while(|part| part != &"}").collect();
                let atlas = format!("{{ {} }}", atlas.join(" "));
                atlas.parse()?
            },
            Some(door) => door.parse()?,
            None => return Err(Self::Err::Syntax)
        };

        Ok(ForwardingStatement{ protocol, address, target })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_method() {
        assert_eq!("GET".parse(), Ok(Method::GET));
        assert_eq!("PUT".parse(), Ok(Method::PUT));
    }

    #[test]
    #[should_panic]
    fn cannot_parse_invalid_method() {
        "CREATE".parse::<Method>().unwrap();
    }

    #[test]
    fn can_parse_map_statement() {
        let actual: MapStatement = "map GET /primes to /var/run/eratosthenes.door".parse().unwrap();
        let expected = MapStatement {
            method: Method::GET,
            prefix: "/primes".parse().unwrap(),
            door: "/var/run/eratosthenes.door".parse().unwrap()
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn can_parse_atlas() {
        let actual: Atlas = r#"{
            map GET /primes to /var/run/eratosthenes.door
            map POST /blog to /var/run/update_blog.door
        }"#.parse().unwrap();
        let maps = vec![
            MapStatement {
                method: Method::GET,
                prefix: "/primes".parse().unwrap(),
                door: "/var/run/eratosthenes.door".parse().unwrap()
            },
            MapStatement{
                method: Method::POST,
                prefix: "/blog".parse().unwrap(),
                door: "/var/run/update_blog.door".parse().unwrap()
            }
        ];
        let expected = Atlas{ maps };
        assert_eq!(actual, expected);
    }

    #[test]
    fn can_parse_forwarding_target() {
        let actual: ForwardingTarget = "/door/path".parse().unwrap();
        let expected = ForwardingTarget::Door("/door/path".parse().unwrap());
        assert_eq!(actual, expected);

        let actual: ForwardingTarget = r#"{
            map GET /primes to /var/run/eratosthenes.door
            map POST /blog to /var/run/update_blog.door
        }"#.parse().unwrap();
        let maps = vec![
            MapStatement {
                method: Method::GET,
                prefix: "/primes".parse().unwrap(),
                door: "/var/run/eratosthenes.door".parse().unwrap()
            },
            MapStatement{
                method: Method::POST,
                prefix: "/blog".parse().unwrap(),
                door: "/var/run/update_blog.door".parse().unwrap()
            }
        ];
        let atlas = Atlas{ maps };
        let expected = ForwardingTarget::Atlas(atlas);
        assert_eq!(actual, expected);
    }

    #[test]
    fn can_parse_protocol() {
        assert_eq!("udp".parse::<Protocol>().unwrap(), Protocol::UDP);
        assert_eq!("tcp".parse::<Protocol>().unwrap(), Protocol::TCP);
        assert_eq!("tls".parse::<Protocol>().unwrap(), Protocol::TLS);
        assert_eq!("http".parse::<Protocol>().unwrap(), Protocol::HTTP);
        assert_eq!("https".parse::<Protocol>().unwrap(), Protocol::HTTPS);
    }

    #[test]
    #[should_panic]
    fn cannot_parse_invalid_protocol() {
        "ICMP".parse::<Protocol>().unwrap();
    }

    #[test]
    fn can_parse_forwarding_statement_with_door() {
        let actual: ForwardingStatement = "forward udp 0.0.0.0:53 to /dns.door".parse().unwrap();
        let target = ForwardingTarget::Door("/dns.door".parse().unwrap());
        let protocol: Protocol = "udp".parse().unwrap();
        let address: SocketAddr = "0.0.0.0:53".parse().unwrap();
        let expected = ForwardingStatement{ protocol, address, target };
        assert_eq!(actual, expected);
    }

    #[test]
    fn can_parse_forwarding_statement_with_atlas() {
        let actual: ForwardingStatement = r#"forward http 0.0.0.0:80 to {
            map GET / to /var/run/acme_client.door
        }"#.parse().unwrap();
        let maps = vec![
            MapStatement {
                method: Method::GET,
                prefix: "/".parse().unwrap(),
                door: "/var/run/acme_client.door".parse().unwrap()
            }
        ];
        let atlas = Atlas{ maps };
        let target = ForwardingTarget::Atlas(atlas);
        let protocol: Protocol = "http".parse().unwrap();
        let address: SocketAddr = "0.0.0.0:80".parse().unwrap();
        let expected = ForwardingStatement{ protocol, address, target };
        assert_eq!(actual, expected);
    }
}
