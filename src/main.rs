use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut listener = TcpListener::bind("127.0.0.1:25565").await?;

    loop {
        let (client_sock, _) = listener.accept().await?;

        tokio::spawn(async move {
            let server_sock = TcpStream::connect("localhost:25566").await.unwrap();

            let (mut client_read, mut client_write) = tokio::io::split(client_sock);
            let (mut server_read, mut server_write) = tokio::io::split(server_sock);

            tokio::spawn(async move {
                let mut buf = [0; 1024];

                loop {
                    let n = match client_read.read(&mut buf).await {
                        Ok(n) if n == 0 => return,
                        Ok(n) => n,
                        Err(e) => {
                            println!("failed to read from socket, err = {:?}", e);
                            return;
                        }
                    };

                    if let Err(e) = server_write.write_all(&buf[0..n]).await {
                        println!("failed to write to socket; err = {:?}", e);
                        return;
                    }
                }
            });

            tokio::spawn(async move {
                let mut buf = [0; 1024];

                loop {
                    let n = match server_read.read(&mut buf).await {
                        Ok(n) if n == 0 => return,
                        Ok(n) => n,
                        Err(e) => {
                            println!("failed to read from socket, err = {:?}", e);
                            return;
                        }
                    };

                    if let Err(e) = client_write.write_all(&buf[0..n]).await {
                        println!("failed to write to socket; err = {:?}", e);
                        return;
                    }
                }
            });
        });
    }
}