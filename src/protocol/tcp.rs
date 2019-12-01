use crate::EndpointParser;
use crate::EndpointSelector;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub struct TcpEndpointParser;

impl EndpointParser<SocketAddr> for TcpEndpointParser {
    fn parse_endpoint(&self, input: String) -> SocketAddr {
        input.parse().unwrap()
    }
}

pub async fn tcp(
    address: SocketAddr,
    mut endpoint_selector: EndpointSelector<SocketAddr>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut listener = TcpListener::bind(address)
        .await
        .expect("Could not bind TCP server");

    loop {
        let (client_sock, _) = listener.accept().await?;
        let endpoint = endpoint_selector.next();

        tokio::spawn(async move {
            let server_sock = TcpStream::connect(endpoint)
                .await
                .expect("Could not connect to endpoint");

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
