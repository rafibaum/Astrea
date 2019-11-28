use crate::AstreaConfig;
use crate::EndpointSelector;
use core::str::FromStr;
use hyper::header::HeaderValue;
use hyper::service::{make_service_fn, service_fn};
use hyper::{header, Body, Client, Error, Request, Server, Uri};
use hyper_tls::HttpsConnector;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

pub async fn http(
    config: AstreaConfig,
    endpoint_selector: Box<dyn EndpointSelector + Send + Sync>,
) -> Result<(), Box<dyn std::error::Error>> {
    let endpoint_selector = Arc::new(Mutex::new(endpoint_selector));
    let https = HttpsConnector::new().expect("TLS initialization failed");
    let client = Arc::new(Client::builder().build::<_, hyper::Body>(https));

    let proxy_service = make_service_fn(move |_| {
        let endpoint_selector = endpoint_selector.clone();
        let client = client.clone();
        async move {
            Ok::<_, Error>(service_fn(move |mut request: Request<Body>| {
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
