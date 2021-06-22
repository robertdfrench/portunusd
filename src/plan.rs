/* 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2021 Robert D. French
 */
//! Configuration commands for PortunusD

use std::net::AddrParseError;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;

pub struct RelayPlan {
    pub network_address: SocketAddr,
    pub application_path: PathBuf,
}

#[derive(Debug)]
pub enum ParseError {
    Network(AddrParseError),
    Syntax,
}

impl From<AddrParseError> for ParseError {
    fn from(other: AddrParseError) -> Self {
        Self::Network(other)
    }
}

impl FromStr for RelayPlan {
    type Err = ParseError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        let mut components = source.split_ascii_whitespace();
        match components.next() {
            Some("forward") => {},
            _ => return Err(Self::Err::Syntax)
        }

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

        Ok(Self{ network_address, application_path })
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
        let plan: RelayPlan = "forward 0.0.0.0:80 to /var/run/hello_web.portunusd".parse().unwrap();
        assert!(plan.network_address.is_ipv4());
        assert_eq!(plan.network_address.port(), 80);
        assert_eq!(plan.network_address.ip(), IpAddr::V4(Ipv4Addr::new(0,0,0,0)));
        assert_eq!(plan.application_path.to_str(), Some("/var/run/hello_web.portunusd"));
    }

    #[test]
    fn supports_ipv6() {
        let plan: RelayPlan = "forward [2001:db8::1]:8080 to /door/path".parse().unwrap();
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
        let _plan: RelayPlan = "foward 0.0.0.0:80 to /var/run/hello_web.portunusd".parse().unwrap();
    }
}
