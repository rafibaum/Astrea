mod endpoint;
mod protocol;

use crate::endpoint::*;
use crate::protocol::http::HttpsConfig;
use crate::protocol::{http, tcp, Protocol};
use serde::Deserialize;
use std::collections::VecDeque;
use std::fs::File;
use std::net::IpAddr;

#[derive(Debug, Deserialize)]
pub struct AstreaConfig {
    host: IpAddr,
    #[serde(default = "default_port")]
    port: u16,
    endpoints: Vec<String>,
    #[serde(rename = "endpoint-selector")]
    endpoint_selector: EndpointSelectors,
    protocol: Protocol,
    #[serde(rename = "https")]
    https_config: Option<HttpsConfig>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_file = File::open("astrea.yml").unwrap();
    let config: AstreaConfig = serde_yaml::from_reader(config_file).unwrap();
    let endpoint_selector: Box<dyn EndpointSelector + Send + Sync> = match config.endpoint_selector
    {
        EndpointSelectors::RoundRobin => {
            Box::new(RoundRobin::new(VecDeque::from(config.endpoints.clone())))
        }
    };

    match config.protocol {
        Protocol::TCP => tcp(config, endpoint_selector).await,
        Protocol::HTTP => http(config, endpoint_selector).await,
    }
}

fn default_port() -> u16 {
    80
}
