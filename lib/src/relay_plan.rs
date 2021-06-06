use std::net::{AddrParseError, IpAddr, SocketAddr};
use std::num::ParseIntError;
use std::path::PathBuf;
use std::str::FromStr;

pub struct RelayPlan {
    network_address: SocketAddr,
    door_address: PathBuf
}

#[derive(Debug)]
pub enum RelayPlanParseError {
    AddrParseError(AddrParseError),
    ParseIntError(ParseIntError),
    MissingComponents
}

impl From<AddrParseError> for RelayPlanParseError {
    fn from(error: AddrParseError) -> Self {
        Self::AddrParseError(error)
    }
}

impl From<ParseIntError> for RelayPlanParseError {
    fn from(error: ParseIntError) -> Self {
        Self::ParseIntError(error)
    }
}

impl std::fmt::Display for RelayPlanParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RelayPlanParseError::AddrParseError(e) => write!(f, "{}", e),
            RelayPlanParseError::ParseIntError(e) => {
                write!(f, "Invalid port: {}", e)
            },
            RelayPlanParseError::MissingComponents => {
                write!(f, "Not all components were specified")
            }
        }
    }
}

impl FromStr for RelayPlan {
    type Err = RelayPlanParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // forward IP_ADDRESS port PORT to PATH
        let components: Vec<&str> = s.split(' ').collect();
        if components.len() < 6 {
            return Err(RelayPlanParseError::MissingComponents)
        }

        let ip_address = components[1].parse::<IpAddr>()?;
        let port = components[3].parse::<u16>()?;

        let network_address = SocketAddr::new(ip_address, port);
        let door_address = PathBuf::from(components[5]);

        Ok(RelayPlan{ network_address, door_address })
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_plan_config_statement() {
        let statement = "forward 0.0.0.0 port 80 to /var/run/app.door";
        let plan = statement.parse::<RelayPlan>().
            expect("Failed to parse");
        assert_eq!(
            plan.door_address.to_str().unwrap(),
            "/var/run/app.door"
        );
        assert_eq!(plan.network_address.port(), 80);
        assert_eq!(plan.network_address.is_ipv4(), true);
    }

    #[test]
    fn parsing_supports_ipv6() {
        let statement = "forward :: port 80 to /var/run/app.door";
        let plan = statement.parse::<RelayPlan>().unwrap();
        assert_eq!(plan.network_address.port(), 80);
        assert_eq!(plan.network_address.is_ipv4(), false);
    }
}
