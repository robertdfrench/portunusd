/* 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2021 Robert D. French
 */
//! Config File Parser
//!
//! This module is responsible for reading portunusd.conf and transforming it into a set of
//! Parameters and Relay Plans that PortunusD can understand. This will determine which ports
//! PortunusD will bind to, what tls certificate it will use, and how http requests will be mapped
//! to different doors.


use std::collections::HashMap;
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

/// Generic Parsing Error
#[derive(Debug,PartialEq)]
pub struct ParseError(String);

macro_rules! parse_error {
    ( $onlystr:expr ) => {
        Err(ParseError($onlystr.to_owned()))
    };
    ( $fmtstr:expr, $($parameters:expr), + ) => {
        Err(ParseError(format!($fmtstr, $($parameters), +)))
    };
}


impl FromStr for Method {
    type Err = ParseError;

    /// Try to parse an HTTP Method
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
            wtf => parse_error!("Unrecognized Method: {}", wtf)
        }
    }
}


/// Express an association between a `(Method,URI)` tuple and a door.
///
/// # Example
///
/// ```portunusd
/// map GET /photos to /var/run/photo_album.door
/// ```
///
/// would become:
///
/// * `method`: "GET"
/// * `prefix`: "/photos"
/// * `door`: "/var/run/photo_album.door"
#[derive(Debug,PartialEq)]
pub struct MapStatement {
    method: Method,
    prefix: PathBuf,
    door: PathBuf
}


impl FromStr for MapStatement {
    type Err = ParseError ;

    /// Attempt to parse a [`MapStatement`]
    fn from_str(input: &str) -> Result<Self,Self::Err> {
        let mut parts = input.split_whitespace();

        if parts.next() != Some("map") {
            return parse_error!("MapStatement should begin with 'map'");
        }

        let method: Method = match parts.next() {
            Some(m) => m.parse()?,
            None => return parse_error!("MapStatement: No method specified")
        };

        let prefix: PathBuf = match parts.next() {
            Some(p) => p.parse().unwrap(), // PathBuf.parse is infallible
            None => return parse_error!("MapStatement: No prefix specified")
        };

        if parts.next() != Some("to") {
            return parse_error!("MapStatement: Needs a 'to door' clause");
        }

        let door: PathBuf = match parts.next() {
            Some(d) => d.parse().unwrap(), // PathBuf.parse is infallible
            None => return parse_error!("MapStatement: No door path specified")
        };

        Ok(Self{ method, prefix, door })
    }
}


/// A collection of maps.
///
/// An http forwarding statement can discriminate on method and URI prefix, so it may have multiple
/// destination doors. The collection of [`MapStatement`]s associated with a forwarding statement
/// is called an Atlas.
///
/// # Example
///
/// ```portunusd
/// forward http 0.0.0.0:80 to {
///     map GET /photos to /var/run/photo_album.door
///     map GET /posts to /var/run/blog_posts.door
/// }
/// ```
///
/// In this example, the parts between the curly braces are the "Atlas".
#[derive(Debug,PartialEq)]
pub struct Atlas {
    maps: Vec<MapStatement>
}

impl FromStr for Atlas {
    type Err = ParseError ;

    /// map GET /primes to /var/run/eratosthenes.door
    fn from_str(input: &str) -> Result<Self,Self::Err> {
        let mut parts = input.split_whitespace();

        if parts.next() != Some("{") {
            return parse_error!("Atlas: Should start with curly brace");
        }

        if parts.nth_back(0) != Some("}") {
            return parse_error!("Atlas: Should end with curly brace");
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


/// Something to which request data can be delivered.
///
/// This is either a door, or an Atlas of Maps to Doors. Either way, it is all we need in order to
/// specify who should receive a given request.
#[derive(Debug,PartialEq)]
pub enum ForwardingTarget {
    Door(PathBuf),
    Atlas(Atlas)
}


impl FromStr for ForwardingTarget {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self,Self::Err> {
        if input.starts_with("/") {
            let door: PathBuf = input.parse().unwrap(); // PathBuf.parse is Infallible
            return Ok(Self::Door(door))
        } else if input.starts_with("{") {
            let atlas: Atlas = input.parse()?;
            return Ok(Self::Atlas(atlas))
        } else {
            return parse_error!("ForwardingTarget should start with '/' or '{{': {}", input);
        }
    }
}


/// Mixed Transmission / Application layer Protocol
///
/// PortunusD needs to know whether to bind to UDP or TCP ports. For TCP ports, it also needs to
/// know whether to use TLS, and (separately) whether to expect HTTP requests (which can have
/// separate routing as defined in an [`Atlas`]).
///
/// # Example
///
/// ```portunusd
/// # domain, certification, and key are needed for TLS
/// set domain portunusd.net
/// set certificate /opt/local/certificates/portunusd.net
/// set key /opt/local/keys/portunusd.net
///
///
/// # RFC 862 Echo Service
///
/// ## UDP Protocol
/// forward udp 0.0.0.0:7 to /var/run/echo.door
///
/// ## TCP Protocol
/// forward tcp 0.0.0.0:7 to /var/run/echo.door
///
/// ## TLS "Protocol" (implies TCP, so we can't re-use port 7)
/// forward tls 0.0.0.0:7777 to /var/run/echo.door
///
///
/// # Force HTTPS Connections
///
/// ## If a user tries to connect via HTTP (i.e., not using TLS), we want to
/// ## redirect them to use HTTPS instead.
/// forward http 0.0.0.0:80 to {
///     map GET / to /var/run/https_redirect.door
///     map POST / to /var/run/https_redirect.door
///     map DELETE / to /var/run/https_redirect.door
/// }
/// 
/// # Mailing List API
///
/// ## Specifying 'https' here implies TCP, HTTP, and TLS. This means that no
/// ## other TCP service can claim port 443. It means that the domain, key, and
/// ## certificate parameters must be set (for tls). Lastly, it means that the
/// ## forwarding target must be an Atlas (a collection of "map" statements).
/// forward https 0.0.0.0:443 to {
/// 	map GET /subscriptions to /var/run/list_subscriptions.door
/// 	map POST /subscriptions/new to /var/run/mailer_signup.door
/// 	map DELETE /subscriptions to /var/run/unsubscribe.door
/// }
/// ```
#[derive(Debug,PartialEq)]
pub enum Protocol {
    UDP,
    TCP,
    /// Implies TCP. Any relay configured with TLS will also require that the `certificate` and
    /// `key` [`Parameter`]s are set.
    TLS,
    /// Implies TCP, and implies that a corresponding [`Atlas`] will be defined.
    HTTP,
    /// Implies HTTP and TLS
    HTTPS
}


impl FromStr for Protocol {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self,Self::Err> {
        match input {
            "udp" => Ok(Self::UDP),
            "tcp" => Ok(Self::TCP),
            "tls" => Ok(Self::TLS),
            "http" => Ok(Self::HTTP),
            "https" => Ok(Self::HTTPS),
            unrecognized => parse_error!("Unrecognized Protocol: {}", unrecognized)
        }
    }
}


/// Tells PortunusD how to handle a network request
///
/// # Example
///
/// ```portunusd
/// forward udp 0.0.0.0:7 to /var/run/echo.door
/// forward http 0.0.0.0:80 to {
/// 	map GET / to /var/run/acme_client.door
/// }
/// ```
///
/// The above would tell PortunusD that any UDP traffic arriving on port 7 should be forwarded to
/// the `/var/run/echo.door` application door. It also states that any TCP traffic arriving on port
/// 80 should be interpreted as HTTP, and forwarded to `/var/run/acme_client.door` if and only if
/// it is a "GET" request whose URI begins with "/".
#[derive(Debug,PartialEq)]
pub struct ForwardingStatement {
    pub protocol: Protocol,
    pub address: SocketAddr,
    pub target: ForwardingTarget
}


impl From<AddrParseError> for ParseError {
    fn from(ape: AddrParseError) -> Self {
        Self(format!("Invalid Address: {}", ape))
    }
}

impl FromStr for ForwardingStatement {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self,Self::Err> {
        let mut parts = input.split_whitespace();

        if parts.next() != Some("forward") {
            return parse_error!("ForwardingStatement should start with 'forward': {}", input);
        }

        let protocol: Protocol = match parts.next() {
            Some(p) => p.parse()?,
            None => return parse_error!("ForwardingStatement missing Protocol: {}", input)
        };

        let address: SocketAddr = match parts.next() {
            Some(a) => a.parse()?,
            None => return parse_error!("ForwardingStatement missing SocketAddr: {}", input)
        };

        if parts.next() != Some("to") {
            return parse_error!("ForwardingStatement needs 'to /door/path': {}", input);
        }

        let target: ForwardingTarget = match parts.next() {
            Some("{") => {
                let atlas: Vec<&str> = parts.take_while(|part| part != &"}").collect();
                let atlas = format!("{{ {} }}", atlas.join(" "));
                atlas.parse()?
            },
            Some(door) => door.parse()?,
            None => return parse_error!("ForwardingStatement missing Target: {}", input)
        };

        Ok(ForwardingStatement{ protocol, address, target })
    }
}

#[derive(Debug,PartialEq)]
pub struct Parameter {
    key: String,
    value: String
}


impl FromStr for Parameter {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self,Self::Err> {
        let mut parts = input.split_whitespace();

        if parts.next() != Some("set") {
            return parse_error!("Parameters must begin with 'set': {}", input);
        }

        let key = match parts.next() {
            Some(key) => key.to_owned(),
            None => return parse_error!("Parameter missing key: {}", input)
        };

        let value = match parts.next() {
            Some(value) => value.to_owned(),
            None => return parse_error!("Parameter missing value: {}", input)
        };

        Ok(Parameter{ key, value })
    }
}

#[derive(Debug,PartialEq)]
pub struct Config {
    parameters: HashMap<String,String>,
    pub statements: Vec<ForwardingStatement>
}


impl FromStr for Config {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self,Self::Err> {
        let mut lines = input.lines();
        let mut parameters = HashMap::new();
        let mut statements = vec![];

        let loop_lines = &mut lines;

        while let Some(line) = loop_lines.next() {
            if line.starts_with("set") {
                let parameter: Parameter = line.parse()?;
                parameters.insert(parameter.key, parameter.value);
            } else if line.starts_with("#") {
                // comment, skip
            } else if line.len() == 0 {
                // empty line, skip
            } else if line.starts_with("forward") {
                if line.ends_with("{") {
                    // this is a block, so keep absorbing until the block ends
                    let block = loop_lines.take_while(|l| l != &"}");
                    let block: Vec<&str> = block.collect();
                    let block = format!("{} {} }}", line, block.join(" "));
                    let statement: ForwardingStatement = block.parse()?;
                    statements.push(statement)
                } else {
                    let statement: ForwardingStatement = line.parse()?;
                    statements.push(statement)
                }
            } else {
                return parse_error!("Unparseable Nonsense: {}", line);
            }
        }

        Ok(Self{ parameters, statements })
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

    #[test]
    fn can_parse_parameter() {
        let actual: Parameter = "set pi 22/7".parse().unwrap();
        let expected = Parameter{
            key: "pi".to_owned(),
            value: "22/7".to_owned()
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn can_parse_file() {
        let config: Config = r#"# Cool comment
set domain example.org
set certificate /opt/local/etc/certificates/example.org.pem
set key /opt/local/etc/certificates/example.org.key

# DNS
forward udp 0.0.0.0:53 to /var/run/dns.door
forward tcp 0.0.0.0:53 to /var/run/dns.door

# Blog
forward http 0.0.0.0:80 to {
    map GET / to /var/run/acme_client.door
}

forward https 0.0.0.0:443 to {
    map GET / to /var/run/blog_content.door
    map POST /signup to /var/run/subscribe.door
    map DELETE /signup to /var/run/unsubscribe.door
}
"#.parse().unwrap();

        assert_eq!(config.parameters.get("domain").unwrap(), "example.org");
        assert_eq!(config.statements[0].protocol, Protocol::UDP);
        assert_eq!(config.statements.len(), 4);
    }
}
