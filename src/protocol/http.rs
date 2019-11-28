use crate::AstreaConfig;
use crate::EndpointSelector;
use core::str::FromStr;
use hyper::client::connect::Connect;
use hyper::error::Result as HyperResult;
use hyper::header::HeaderValue;
use hyper::server::conn::Http;
use hyper::service::service_fn;
use hyper::{header, Body, Client, Request, Response, Uri};
use hyper_tls::{HttpsConnector, MaybeHttpsStream};
use native_tls::Identity;
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio_tls::TlsAcceptor;

#[derive(Debug, Deserialize)]
pub struct HttpsConfig {
    #[serde(rename = "identity-file")]
    identity_file: String,
    #[serde(default)]
    password: String,
}

pub async fn http(
    config: AstreaConfig,
    endpoint_selector: Box<dyn EndpointSelector + Send + Sync>,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = Arc::new(config);
    let https_connector = HttpsConnector::new().expect("TLS initialization failed");
    let client = Arc::new(Client::builder().build::<_, hyper::Body>(https_connector));
    let endpoint_selector = Arc::new(Mutex::new(endpoint_selector));
    
    https(config, client.clone(), endpoint_selector.clone()).await
}

async fn https<C: Connect + 'static>(
    config: Arc<AstreaConfig>,
    client: Arc<Client<C, hyper::Body>>,
    endpoint_selector: Arc<Mutex<Box<dyn EndpointSelector + Send + Sync>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let https_config = config.https_config.as_ref().unwrap();
    let mut file = File::open(https_config.identity_file.clone()).unwrap();
    let mut identity = vec![];
    file.read_to_end(&mut identity).unwrap();
    let identity = Identity::from_pkcs12(&identity, &https_config.password).unwrap();
    let acceptor = Arc::new(TlsAcceptor::from(
        native_tls::TlsAcceptor::new(identity).unwrap(),
    ));

    let server = Arc::new(Http::new());

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

            let proxy_service = service_fn(move |request: Request<Body>| {
                proxy_request(request, endpoint_selector.clone(), client.clone())
            });

            server
                .serve_connection(https_stream, proxy_service)
                .await
                .unwrap();
        });
    }
}

async fn proxy_request<C: Connect + 'static>(
    mut request: Request<Body>,
    endpoint_selector: Arc<Mutex<Box<dyn EndpointSelector + Send + Sync>>>,
    client: Arc<Client<C, hyper::Body>>,
) -> HyperResult<Response<Body>> {
    let endpoint = { Uri::from_str(&endpoint_selector.lock().unwrap().next()).unwrap() };
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
