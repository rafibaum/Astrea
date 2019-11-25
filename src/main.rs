use serde::Deserialize;
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_file = File::open("astrea.yml").unwrap();
    let config: AstreaConfig = serde_yaml::from_reader(config_file).unwrap();

    let mut listener = TcpListener::bind((config.host, config.port)).await?;

    loop {
        let (client_sock, _) = listener.accept().await?;
        let endpoint = *config.endpoints.first().unwrap();

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
    let mut buf = [0; 1024];

    loop {
        let n = match reader.read(&mut buf).await {
            Ok(n) if n == 0 => return,
            Ok(n) => n,
            Err(e) => {
                println!("failed to read from socket, err = {:?}", e);
                return;
            }
        };

        if let Err(e) = writer.write_all(&buf[0..n]).await {
            println!("failed to write to socket; err = {:?}", e);
            return;
        }
    }
}
