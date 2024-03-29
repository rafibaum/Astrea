use crate::EndpointParser;
use crate::EndpointSelector;
use core::str::FromStr;
use hyper::client::connect::Connect;
use hyper::error::Result as HyperResult;
use hyper::header::HeaderValue;
use hyper::server::conn::Http;
use hyper::service::{make_service_fn, service_fn};
use hyper::{header, Body, Client, Error, Request, Response, Server, Uri};
use hyper_tls::{HttpsConnector, MaybeHttpsStream};
use native_tls::Identity;
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio_tls::TlsAcceptor;

#[derive(Debug, Deserialize)]
pub struct HttpsConfig {
    #[serde(rename = "identity-file")]
    identity_file: String,
    #[serde(default)]
    password: String,
    #[serde(default = "default_port")]
    port: u16,
}

pub struct HttpEndpointParser;

impl EndpointParser<Uri> for HttpEndpointParser {
    fn parse_endpoint(&self, input: String) -> Uri {
        Uri::from_str(&input).expect("Could not convert endpoint to URL")
    }
}

pub async fn http(
    address: SocketAddr,
    endpoint_selector: EndpointSelector<Uri>,
    https_config: Option<HttpsConfig>,
) -> Result<(), Box<dyn std::error::Error>> {
    let address = Arc::new(address);
    let https_connector = HttpsConnector::new().expect("TLS initialization failed");
    let client = Arc::new(Client::builder().build::<_, hyper::Body>(https_connector));
    let endpoint_selector = Arc::new(Mutex::new(endpoint_selector));

    if https_config.is_some() {
        let client = client.clone();
        let endpoint_selector = endpoint_selector.clone();
        let address = address.clone();
        tokio::spawn(async move {
            https(address, https_config, client, endpoint_selector)
                .await
                .unwrap();
        });
    }

    let http_service = make_service_fn(|_| {
        let client = client.clone();
        let endpoint_selector = endpoint_selector.clone();
        async {
            Ok::<_, Error>(service_fn(move |request: Request<Body>| {
                proxy_request(request, endpoint_selector.clone(), client.clone())
            }))
        }
    });

    let server = Server::bind(&address).serve(http_service);

    if let Err(e) = server.await {
        Err(Box::new(e) as Box<dyn std::error::Error>)
    } else {
        Ok(())
    }
}

async fn https<C: Connect + 'static>(
    address: Arc<SocketAddr>,
    https_config: Option<HttpsConfig>,
    client: Arc<Client<C, hyper::Body>>,
    endpoint_selector: Arc<Mutex<EndpointSelector<Uri>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let https_config = https_config.as_ref().unwrap();

    let mut file = File::open(https_config.identity_file.clone())
        .expect("TLS certificate file could not be opened");
    let mut identity = vec![];
    file.read_to_end(&mut identity)
        .expect("TLS certificate file could not be read");
    let identity = Identity::from_pkcs12(&identity, &https_config.password)
        .expect("Incorrect password used for TLS certificate");

    let acceptor = Arc::new(TlsAcceptor::from(
        native_tls::TlsAcceptor::new(identity).expect("Could not initialise TLS handler"),
    ));
    let server = Arc::new(Http::new());
    let mut address = address.as_ref().clone();
    address.set_port(https_config.port);

    let mut listener = TcpListener::bind(address)
        .await
        .expect("Could not bind TCP listener");

    loop {
        let acceptor = acceptor.clone();
        let endpoint_selector = endpoint_selector.clone();
        let client = client.clone();
        let server = server.clone();

        let (client_sock, _) = listener.accept().await?;

        tokio::spawn(async move {
            let secure_sock = acceptor
                .accept(client_sock)
                .await
                .expect("Could not negotiate TLS stream");
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
    endpoint_selector: Arc<Mutex<EndpointSelector<Uri>>>,
    client: Arc<Client<C, hyper::Body>>,
) -> HyperResult<Response<Body>> {
    let endpoint = { endpoint_selector.lock().unwrap().next() };
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

fn default_port() -> u16 {
    443
}
