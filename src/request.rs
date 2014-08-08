//! Iron's HTTP Request representation and associated methods.

use std::io::net::ip::SocketAddr;
use http::server::request::{AbsoluteUri, AbsolutePath};
use http::headers::request::HeaderCollection;
use http::method::Method;
use url::Url;
pub use HttpRequest = http::server::request::Request;

use super::alloy::Alloy;

/// The `Request` given to all `Middleware`.
///
/// Stores all the properties of the client's request plus
/// an `Alloy` for data communication between middleware.
///
pub struct Request {
    /// The requested url as a `url::Url`.
    ///
    /// See `servo/rust-url`'s documentation for more information.  
    /// Useful methods include `Url::host`, `Url::domain` and `Url::query_pairs`.
    pub url: Url,

    /// The originating address of the request.
    pub remote_addr: Option<SocketAddr>,

    /// The request headers.
    pub headers: Box<HeaderCollection>,

    /// The request body.
    pub body: String,

    /// The request method.
    pub method: Method,

    /// Storage for data passed between middleware.
    pub alloy: Alloy
}

impl Request {
    /// Create a request from an HttpRequest.
    ///
    /// This constructor consumes the HttpRequest.
    pub fn from_http(req: HttpRequest) -> Option<Request> {
        match req.request_uri {
            AbsoluteUri(url) => {
                Some(Request {
                    url: url,
                    remote_addr: req.remote_addr,
                    headers: req.headers,
                    body: req.body,
                    method: req.method,
                    alloy: Alloy::new()
                })
            },
            AbsolutePath(path) => {
                // Attempt to prepend the Host header (mandatory in HTTP/1.1)
                // XXX: HTTPS incompatible, update when switching to Teepee.
                let url_string = match req.headers.host {
                    Some(ref host) => format!("http://{}{}", host, path),
                    None => return None
                };

                let url = match Url::parse(url_string.as_slice()) {
                    Ok(url) => url,
                    Err(_) => return None // Very unlikely.
                };

                Some(Request {
                    url: url,
                    remote_addr: req.remote_addr,
                    headers: req.headers,
                    body: req.body,
                    method: req.method,
                    alloy: Alloy::new()
                })
            },
            _ => None
        }
    }
}
