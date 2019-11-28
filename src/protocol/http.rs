use crate::AstreaConfig;
use crate::EndpointSelector;
use core::str::FromStr;
use hyper::header::HeaderValue;
use hyper::server::conn::Http;
use hyper::service::service_fn;
use hyper::{header, Body, Client, Request, Uri};
use hyper_tls::{HttpsConnector, MaybeHttpsStream};
use native_tls::Identity;
use std::fs::File;
use std::io::Read;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio_tls::TlsAcceptor;

pub async fn http(
    config: AstreaConfig,
    endpoint_selector: Box<dyn EndpointSelector + Send + Sync>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Initialise server-side HTTPS
    let https = HttpsConnector::new().expect("TLS initialization failed");
    let mut file = File::open("astrea.p12").unwrap();
    let mut identity = vec![];
    file.read_to_end(&mut identity).unwrap();
    let identity = Identity::from_pkcs12(&identity, "astrea").unwrap();
    let acceptor = Arc::new(TlsAcceptor::from(
        native_tls::TlsAcceptor::new(identity).unwrap(),
    ));

    let server = Arc::new(Http::new());
    let client = Arc::new(Client::builder().build::<_, hyper::Body>(https));
    let endpoint_selector = Arc::new(Mutex::new(endpoint_selector));

    let mut listener = TcpListener::bind((config.host, config.port)).await?;

    loop {
        let (client_sock, _) = listener.accept().await?;

        let acceptor = acceptor.clone();
        let endpoint_selector = endpoint_selector.clone();
        let client = client.clone();
        let server = server.clone();

        tokio::spawn(async move {
            let secure_sock = acceptor.accept(client_sock).await.unwrap();
            let https_stream = MaybeHttpsStream::Https(secure_sock);

            let proxy_service = service_fn(move |mut request: Request<Body>| {
                let endpoint = Uri::from_str(&endpoint_selector.lock().unwrap().next()).unwrap();
                let client = client.clone();
                async move {
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
            });

            server
                .serve_connection(https_stream, proxy_service)
                .await
                .unwrap();
        });
    }
}
