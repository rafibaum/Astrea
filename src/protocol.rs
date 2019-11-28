use crate::endpoint::EndpointSelector;
use crate::AstreaConfig;
use core::str::FromStr;
use hyper::header::HeaderValue;
use hyper::service::{make_service_fn, service_fn};
use hyper::{header, Body, Client, Error, Request, Server, Uri};
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[derive(Debug, Deserialize)]
pub enum Protocol {
    #[serde(alias = "http")]
    HTTP,
    #[serde(alias = "tcp")]
    TCP,
}

pub async fn http(
    config: AstreaConfig,
    endpoint_selector: Box<dyn EndpointSelector + Send + Sync>,
) -> Result<(), Box<dyn std::error::Error>> {
    let endpoint_selector = Arc::new(Mutex::new(endpoint_selector));
    let proxy_service = make_service_fn(move |_| {
        let endpoint_selector = endpoint_selector.clone();
        async move {
            Ok::<_, Error>(service_fn(move |mut request: Request<Body>| {
                let endpoint = Uri::from_str(&endpoint_selector.lock().unwrap().next()).unwrap();
                async move {
                    let client = Client::new();
                    // Add new endpoint to request
                    let mut uri_parts = request.uri().clone().into_parts();
                    let endpoint_parts = endpoint.into_parts();
                    uri_parts.authority = endpoint_parts.authority;
                    uri_parts.scheme = endpoint_parts.scheme;
                    let uri = Uri::from_parts(uri_parts).unwrap();
                    *request.uri_mut() = uri;

                    // Replace host header value
                    let host_string = &request.uri().authority_part().unwrap().to_string();
                    request
                        .headers_mut()
                        .insert(header::HOST, HeaderValue::from_str(host_string).unwrap());

                    client.request(request).await
                }
            }))
        }
    });

    let server = Server::bind(&SocketAddr::from((config.host, config.port))).serve(proxy_service);

    if let Err(e) = server.await {
        Err(Box::new(e) as Box<dyn std::error::Error>)
    } else {
        Ok(())
    }
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
