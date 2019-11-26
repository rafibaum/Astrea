use crate::endpoint::EndpointSelector;
use crate::AstreaConfig;
use serde::Deserialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[derive(Debug, Deserialize)]
pub enum Protocol {
    #[serde(alias = "tcp")]
    TCP,
}

pub async fn tcp(
    config: AstreaConfig,
    mut endpoint_selector: Box<dyn EndpointSelector>,
) -> Result<(), Box<dyn std::error::Error>> {
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
