use serde::Deserialize;
use std::collections::VecDeque;
use std::fs::File;
use std::net::IpAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::net::TcpStream;

#[derive(Debug, Deserialize)]
struct AstreaConfig {
    host: IpAddr,
    port: u16,
    endpoints: Vec<(IpAddr, u16)>,
    #[serde(rename = "endpoint-selector")]
    endpoint_selector: EndpointSelectors,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_file = File::open("astrea.yml").unwrap();
    let config: AstreaConfig = serde_yaml::from_reader(config_file).unwrap();
    let mut endpoint_selector: Box<dyn EndpointSelector> = match config.endpoint_selector {
        EndpointSelectors::RoundRobin => {
            Box::new(RoundRobin::new(VecDeque::from(config.endpoints)))
        }
    };

    let mut listener = TcpListener::bind((config.host, config.port)).await?;

    loop {
        let (client_sock, _) = listener.accept().await?;
        let endpoint = endpoint_selector.next();

        tokio::spawn(async move {
            let server_sock = TcpStream::connect(endpoint).await.unwrap();

            let (client_read, client_write) = tokio::io::split(client_sock);
            let (server_read, server_write) = tokio::io::split(server_sock);

            tokio::spawn(copy(client_read, server_write));
            tokio::spawn(copy(server_read, client_write));
        });
    }
}

async fn copy<R, W>(mut reader: R, mut writer: W)
where
    R: AsyncReadExt + Unpin,
    W: AsyncWriteExt + Unpin,
{
    if let Err(e) = reader.copy(&mut writer).await {
        println!("Socket broken: {:?}", e);
    }
}

#[derive(Debug, Deserialize)]
enum EndpointSelectors {
    #[serde(alias = "round robin")]
    RoundRobin,
}

trait EndpointSelector {
    fn next(&mut self) -> (IpAddr, u16);
}

struct RoundRobin {
    endpoints: VecDeque<(IpAddr, u16)>,
}

impl RoundRobin {
    fn new(endpoints: VecDeque<(IpAddr, u16)>) -> RoundRobin {
        assert!(!endpoints.is_empty());
        RoundRobin { endpoints }
    }
}

impl EndpointSelector for RoundRobin {
    fn next(&mut self) -> (IpAddr, u16) {
        let result = self.endpoints.pop_front().unwrap();
        self.endpoints.push_back(result);
        result
    }
}