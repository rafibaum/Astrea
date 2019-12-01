mod endpoint;
mod protocol;

use crate::endpoint::*;
use crate::protocol::http::HttpsConfig;
use crate::protocol::{http, tcp, Protocol};
use serde::Deserialize;
use std::fs::File;
use std::net::IpAddr;
use std::net::SocketAddr;

#[derive(Debug, Deserialize)]
pub struct AstreaConfig {
    host: IpAddr,
    #[serde(default = "default_port")]
    port: u16,
    endpoints: Vec<String>,
    #[serde(rename = "endpoint-selector")]
    endpoint_selector: endpoint::Strategy,
    protocol: Protocol,
    #[serde(rename = "https")]
    https_config: Option<HttpsConfig>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_file = File::open("astrea.yml").expect("Config file couldn't be opened");
    let config: AstreaConfig =
        serde_yaml::from_reader(config_file).expect("Config file is incorrectly formatted");
    let address = SocketAddr::from((config.host, config.port));

    let strategy = match config.endpoint_selector {
        Strategy::RoundRobin => Box::new(RoundRobin{})
    };

    match config.protocol {
        Protocol::TCP => {
            tcp(
                address,
                EndpointSelector::new(config.endpoints, tcp::TcpEndpointParser {}, strategy),
            )
            .await
        }
        Protocol::HTTP => {
            http(
                address,
                EndpointSelector::new(config.endpoints, http::HttpEndpointParser {}, strategy),
                config.https_config,
            )
            .await
        }
    }
}

fn default_port() -> u16 {
    80
}
