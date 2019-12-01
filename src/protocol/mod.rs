pub mod http;
pub mod tcp;

pub use self::http::http;
pub use self::tcp::tcp;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub enum Protocol {
    #[serde(rename = "http")]
    HTTP,
    #[serde(rename = "tcp")]
    TCP,
}
